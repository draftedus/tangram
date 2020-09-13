use crate::{
	common::user::{authorize_user, authorize_user_for_organization, User},
	error::Error,
	Context,
};
use anyhow::Result;
use bytes::Buf;
use chrono::prelude::*;
use hyper::{header, Body, Request, Response, StatusCode};
use multer::Multipart;
use sqlx::prelude::*;
use tangram_core::util::id::Id;

#[derive(serde::Serialize)]
struct Props {
	error: Option<String>,
	title: Option<String>,
	owner: Option<String>,
	owners: Option<Vec<Owner>>,
}

#[derive(serde::Serialize)]
struct Owner {
	value: String,
	title: String,
}

pub async fn get(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let props = props(&mut db, user, None, None, None).await?;
	let html = context.pinwheel.render_with("/repos/new", props)?;
	db.commit().await?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

async fn props(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: Option<User>,
	error: Option<String>,
	title: Option<String>,
	owner: Option<String>,
) -> Result<Props> {
	if let Some(user) = user {
		let mut owners = vec![Owner {
			value: format!("user:{}", user.id),
			title: user.email,
		}];
		let rows = sqlx::query(
			"
				select
					organizations.id,
					organizations.name
				from organizations
				join organizations_users
					on organizations_users.organization_id = organizations.id
					and organizations_users.user_id = ?1
			",
		)
		.bind(&user.id.to_string())
		.fetch_all(&mut *db)
		.await?;
		for row in rows {
			let id: String = row.get(0);
			let title: String = row.get(1);
			owners.push(Owner {
				value: format!("organization:{}", id),
				title,
			})
		}
		Ok(Props {
			owners: Some(owners),
			error,
			owner,
			title,
		})
	} else {
		Ok(Props {
			owners: None,
			error,
			owner,
			title,
		})
	}
}

pub async fn post(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let boundary = request
		.headers()
		.get(header::CONTENT_TYPE)
		.and_then(|ct| ct.to_str().ok())
		.and_then(|ct| multer::parse_boundary(ct).ok())
		.ok_or_else(|| Error::BadRequest)?;
	let mut title: Option<String> = None;
	let mut owner: Option<String> = None;
	let mut file_data: Option<Vec<u8>> = None;
	let mut multipart = Multipart::new(request.into_body(), boundary);
	while let Some(mut field) = multipart.next_field().await? {
		let name = field
			.name()
			.map(|name| name.to_string())
			.ok_or(Error::BadRequest)?;
		let mut field_data = Vec::new();
		while let Some(chunk) = field.chunk().await? {
			field_data.extend(chunk.bytes());
		}
		match name.as_str() {
			"title" => {
				title = Some(String::from_utf8(field_data).map_err(|_| Error::BadRequest)?);
			}
			"owner" => {
				owner = Some(String::from_utf8(field_data).map_err(|_| Error::BadRequest)?);
			}
			"file" => {
				file_data = Some(field_data);
			}
			_ => {}
		}
	}
	let title = title.ok_or(Error::BadRequest)?;
	let owner = if context.options.auth_enabled {
		Some(owner.ok_or(Error::BadRequest)?)
	} else {
		None
	};
	let file_data = file_data.ok_or(Error::BadRequest)?;

	let model = match tangram_core::model::Model::from_slice(&file_data) {
		Ok(model) => model,
		Err(e) => {
			dbg!(e);
			let error =
				"The model you uploaded failed to deserialize. Are you sure it is a .tangram file?";
			let props = props(&mut db, user, Some(String::from(error)), Some(title), owner).await?;
			let html = context.pinwheel.render_with("/repos/new", props)?;
			let response = Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(Body::from(html))
				.unwrap();
			return Ok(response);
		}
	};
	let now = Utc::now().timestamp();
	let repo_id = Id::new();
	let result = if let Some(owner) = &owner {
		let owner_parts: Vec<&str> = owner.split(':').collect();
		let owner_type = owner_parts.get(0).ok_or(Error::BadRequest)?;
		let owner_id: Id = owner_parts
			.get(1)
			.ok_or(Error::BadRequest)?
			.parse()
			.map_err(|_| Error::BadRequest)?;
		match *owner_type {
			"user" => create_user_repo(&mut db, owner_id, repo_id, now, &title).await,
			"organization" => {
				if !authorize_user_for_organization(&mut db, user.as_ref().unwrap(), owner_id)
					.await?
				{
					return Err(Error::Unauthorized.into());
				}
				create_org_repo(&mut db, owner_id, repo_id, now, title.as_str()).await
			}
			_ => return Err(Error::BadRequest.into()),
		}
	} else {
		create_root_repo(&mut db, repo_id, now, title.as_str()).await
	};

	if result.is_err() {
		let error = "There was an error uploading your model.";
		let props = props(&mut db, user, Some(String::from(error)), Some(title), owner).await?;
		let html = context.pinwheel.render_with("/repos/new", props)?;
		db.commit().await?;
		let response = Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.body(Body::from(html))
			.unwrap();
		return Ok(response);
	}

	let result = sqlx::query(
		"
			insert into models
				(id, repo_id, created_at, data)
			values (
				?, ?, ?, ?
			)
		",
	)
	.bind(&model.id().to_string())
	.bind(&repo_id.to_string())
	.bind(&now)
	.bind(&base64::encode(&file_data))
	.execute(&mut *db)
	.await;
	if result.is_err() {
		let error = "This model has already been uploaded. Tangram models have unique identifiers and can only belong to one account.";
		let props = props(&mut db, user, Some(String::from(error)), Some(title), owner).await?;
		let html = context.pinwheel.render_with("/repos/new", props)?;
		let response = Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.body(Body::from(html))
			.unwrap();
		return Ok(response);
	};

	db.commit().await?;
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(
			header::LOCATION,
			format!("/repos/{}/models/{}/", repo_id, model.id()),
		)
		.body(Body::empty())
		.unwrap();
	Ok(response)
}

async fn create_user_repo(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user_id: Id,
	repo_id: Id,
	now: i64,
	title: &str,
) -> std::result::Result<sqlx::any::AnyDone, sqlx::error::Error> {
	sqlx::query(
		"
			insert into repos (
				id, created_at, title, user_id
			) values (
				?, ?, ?, ?
			)
		",
	)
	.bind(&repo_id.to_string())
	.bind(&now)
	.bind(&title)
	.bind(&user_id.to_string())
	.execute(&mut *db)
	.await
}

async fn create_org_repo(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	org_id: Id,
	repo_id: Id,
	now: i64,
	title: &str,
) -> std::result::Result<sqlx::any::AnyDone, sqlx::error::Error> {
	sqlx::query(
		"
			insert into repos (
				id, created_at, title, organization_id
			) values (
				?, ?, ?, ?
			)
		",
	)
	.bind(&repo_id.to_string())
	.bind(&now)
	.bind(&title)
	.bind(&org_id.to_string())
	.execute(&mut *db)
	.await
}

async fn create_root_repo(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	repo_id: Id,
	now: i64,
	title: &str,
) -> std::result::Result<sqlx::any::AnyDone, sqlx::error::Error> {
	sqlx::query(
		"
			insert into repos (
				id, created_at, title
			) values (
				?, ?, ?
			)
		",
	)
	.bind(&repo_id.to_string())
	.bind(&now)
	.bind(&title)
	.execute(&mut *db)
	.await
}
