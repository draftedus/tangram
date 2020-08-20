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

struct Options {
	user: User,
	form: Option<Form>,
	error: Option<String>,
}

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
	pub file: String,
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
	let user = authorize_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let response = render(
		&mut db,
		context,
		Options {
			user,
			form: None,
			error: None,
		},
	)
	.await;
	db.commit().await?;
	response
}

async fn render(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: &Context,
	options: Options,
) -> Result<Response<Body>> {
	let props = props(db, options.user, options.form, options.error).await?;
	let html = context.pinwheel.render_with("/repos/new", props)?;
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
	let mut owners = vec![Owner {
		value: format!("user:{}", user.id),
		title: user.email,
	}];
	rows.iter().for_each(|row| {
		let id: String = row.get(0);
		let title: String = row.get(1);
		owners.push(Owner {
			value: format!("organization:{}", id),
			title,
		})
	});
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
	let mut file: Option<String> = None;
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
				file = Some(field.file_name().ok_or(Error::BadRequest)?.to_string());
				file_data = Some(field_data);
			}
			_ => {}
		}
	}
	let title = title.ok_or(Error::BadRequest)?;
	let owner = owner.ok_or(Error::BadRequest)?;
	let file = file.ok_or(Error::BadRequest)?;
	let file_data = file_data.ok_or(Error::BadRequest)?;
	let form = Form { title, owner, file };

	let model = match tangram_core::types::Model::from_slice(&file_data) {
		Ok(model) => model,
		Err(_) => {
			return render(
				&mut db,
				context,
				Options {
					error: Some(String::from("failed to parse model")),
					form: Some(form),
					user,
				},
			)
			.await;
		}
	};
	let now = Utc::now().timestamp();
	let repo_id = Id::new();

	let id_parts: Vec<&str> = form.owner.split(':').collect();

	match id_parts[0] {
		"user" => {
			let user_id: Id = id_parts[1].parse().map_err(|_| Error::BadRequest)?;
			if user_id != user.id {
				return Err(Error::Unauthorized.into());
			}
			let result = sqlx::query(
				"
					insert into repos (
						id, created_at, title, user_id
					) values (
						?1, ?2, ?3, ?4
					)
				",
			)
			.bind(&repo_id.to_string())
			.bind(&now)
			.bind(&form.title)
			.bind(&user_id.to_string())
			.execute(&mut *db)
			.await;
			if result.is_err() {
				return render(
					&mut db,
					context,
					Options {
						error: Some(String::from("A repo with this title already exists.")),
						form: Some(form),
						user,
					},
				)
				.await;
			}
		}
		"organization" => {
			let organization_id: Id = id_parts[1].parse().map_err(|_| Error::BadRequest)?;
			if !authorize_user_for_organization(&mut db, &user, organization_id).await? {
				return Err(Error::Unauthorized.into());
			}
			let result = sqlx::query(
				"
					insert into repos (
						id, created_at, title, organization_id
					) values (
						?1, ?2, ?3, ?4
					)
				",
			)
			.bind(&repo_id.to_string())
			.bind(&now)
			.bind(&form.title)
			.bind(&organization_id.to_string())
			.execute(&mut *db)
			.await;
			if result.is_err() {
				return render(
					&mut db,
					context,
					Options {
						error: Some(String::from(
							"A repo in this organization with this title already exists.",
						)),
						form: Some(form),
						user,
					},
				)
				.await;
			};
		}
		&_ => return Err(Error::BadRequest.into()),
	};

	let result = sqlx::query(
		"
			insert into models
				(id, repo_id, created_at, data)
			values
				(?1, ?2, ?3, ?4)
		",
	)
	.bind(&model.id().to_string())
	.bind(&repo_id.to_string())
	.bind(&now)
	.bind(&base64::encode(&form.file))
	.execute(&mut *db)
	.await;
	if result.is_err() {
		return render(
			&mut db,
			context,
			Options {
				error: Some(String::from("This model has already been uploaded. Tangram models have unique identifiers and can only belong to one account.")),
				form: Some(form),
				user,
			},
		)
		.await;
	};

	db.commit().await?;
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(
			header::LOCATION,
			format!("/repos/{}/models/{}/", repo_id, model.id().to_string()),
		)
		.body(Body::empty())
		.unwrap();
	Ok(response)
}
