use crate::{
	error::Error,
	user::{authorize_user, authorize_user_for_organization, User},
	Context,
};
use anyhow::Result;
use bytes::Buf;
use chrono::prelude::*;
use hyper::{header, Body, Request, Response, StatusCode};
use multer::Multipart;
use sqlx::prelude::*;
use tangram_core::id::Id;

#[derive(serde::Serialize)]
struct Props {
	error: Option<String>,
	form: Option<Form>,
	owners: Vec<Owner>,
}

#[derive(serde::Serialize)]
pub struct Form {
	pub title: String,
	pub owner: String,
}

#[derive(serde::Serialize)]
struct Owner {
	value: String,
	title: String,
}

enum OwnerType {
	User,
	Organization,
}

pub async fn get(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let props = props(&mut db, user, None, None).await?;
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
	user: User,
	form: Option<Form>,
	error: Option<String>,
) -> Result<Props> {
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
		owners,
		error,
		form,
	})
}

pub async fn post(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;

	let boundary = request
		.headers()
		.get(header::CONTENT_TYPE)
		.and_then(|ct| ct.to_str().ok())
		.and_then(|ct| multer::parse_boundary(ct).ok())
		.ok_or(Error::BadRequest)?;
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
	let owner = owner.ok_or(Error::BadRequest)?;
	let file_data = file_data.ok_or(Error::BadRequest)?;
	let form = Form { title, owner };

	let model = match tangram_core::types::Model::from_slice(&file_data) {
		Ok(model) => model,
		Err(_) => {
			let error =
				"The model you uploaded failed to deserialize. Are you sure it is a .tangram file?";
			let props = props(&mut db, user, Some(form), Some(String::from(error))).await?;
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

	let mut user_id = None;
	let mut organization_id = None;
	let owner_parts: Vec<&str> = form.owner.split(':').collect();
	let owner_type = owner_parts.get(0).ok_or(Error::BadRequest)?;
	let owner_type = match *owner_type {
		"user" => OwnerType::User,
		"organization" => OwnerType::Organization,
		_ => return Err(Error::BadRequest.into()),
	};
	let owner_id: Id = owner_parts
		.get(1)
		.ok_or(Error::BadRequest)?
		.parse()
		.map_err(|_| Error::BadRequest)?;
	match owner_type {
		OwnerType::User => {
			if owner_id != user.id {
				return Err(Error::Unauthorized.into());
			}
			user_id = Some(owner_id);
		}
		OwnerType::Organization => {
			if !authorize_user_for_organization(&mut db, &user, owner_id).await? {
				return Err(Error::Unauthorized.into());
			}
			organization_id = Some(owner_id);
		}
	};
	let result = sqlx::query(
		"
			insert into repos (
				id, created_at, title, user_id, organization_id
			) values (
				?, ?, ?, ?, ?
			)
		",
	)
	.bind(&repo_id.to_string())
	.bind(&now)
	.bind(&form.title)
	.bind(&user_id.map(|id| id.to_string()))
	.bind(&organization_id.map(|id| id.to_string()))
	.execute(&mut *db)
	.await;
	if result.is_err() {
		let error = "A repo with this title already exists.";
		let props = props(&mut db, user, Some(form), Some(String::from(error))).await?;
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
		let props = props(&mut db, user, Some(form), Some(String::from(error))).await?;
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
