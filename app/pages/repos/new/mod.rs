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
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use tangram_core::id::Id;

pub async fn get(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let response = render(&mut db, context, user, None).await;
	db.commit().await?;
	return response;
}

async fn render(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: &Context,
	user: User,
	options: Option<Options>,
) -> Result<Response<Body>> {
	let props = props(db, user, options).await?;
	let html = context.pinwheel.render_with("/repos/new", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	return Ok(response);
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	owners: Vec<Owner>,
	error: Option<String>,
	title: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Owner {
	id: String,
	title: String,
}

struct Options {
	title: Option<String>,
	error: String,
}

async fn props(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: User,
	options: Option<Options>,
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
	let mut items = vec![Owner {
		id: format!("user:{}", user.id),
		title: user.email,
	}];
	rows.iter().for_each(|row| {
		let id: String = row.get(0);
		let title: String = row.get(1);
		items.push(Owner {
			id: format!("organization:{}", id),
			title,
		})
	});
	Ok(Props {
		owners: items,
		error: options.as_ref().map(|s| s.error.to_owned()),
		title: options.as_ref().and_then(|s| s.title.to_owned()),
	})
}

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Action {
	pub data: String,
	pub organization_id: Option<String>,
	pub title: String,
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
	let boundary = match request
		.headers()
		.get(header::CONTENT_TYPE)
		.and_then(|ct| ct.to_str().ok())
		.and_then(|ct| multer::parse_boundary(ct).ok())
	{
		Some(boundary) => boundary,
		None => {
			return render(
				&mut db,
				context,
				user,
				Some(Options {
					title: None,
					error: "Failed to parse request body.".into(),
				}),
			)
			.await
		}
	};
	let mut title: Option<String> = None;
	let mut owner_id: Option<String> = None;
	let mut file: Option<Vec<u8>> = None;
	let mut multipart = Multipart::new(request.into_body(), boundary);
	while let Some(mut field) = multipart.next_field().await? {
		let name = match field.name() {
			Some(name) => name.to_owned(),
			None => {
				return render(
					&mut db,
					context,
					user,
					Some(Options {
						title: None,
						error: "Failed to parse request body.".into(),
					}),
				)
				.await
			}
		};
		let mut field_data = Vec::new();
		while let Some(chunk) = field.chunk().await? {
			field_data.extend(chunk.bytes());
		}
		match name.as_str() {
			"title" => {
				title = match String::from_utf8(field_data) {
					Ok(title) => Some(title),
					Err(_) => {
						return render(
							&mut db,
							context,
							user,
							Some(Options {
								title: None,
								error: "Failed to parse request body.".into(),
							}),
						)
						.await
					}
				}
			}
			"owner_id" => {
				owner_id = match String::from_utf8(field_data) {
					Ok(owner_id) => Some(owner_id),
					Err(_) => {
						return render(
							&mut db,
							context,
							user,
							Some(Options {
								title: None,
								error: "Failed to parse request body.".into(),
							}),
						)
						.await
					}
				}
			}
			"file" => file = Some(field_data),
			_ => {
				return render(
					&mut db,
					context,
					user,
					Some(Options {
						title: None,
						error: "Failed to parse request body.".into(),
					}),
				)
				.await
			}
		}
	}

	let title = match title {
		Some(title) => title,
		None => {
			return render(
				&mut db,
				context,
				user,
				Some(Options {
					title: None,
					error: "A title is required.".into(),
				}),
			)
			.await
		}
	};
	let owner_id = match owner_id {
		Some(owner_id) => owner_id,
		None => {
			return render(
				&mut db,
				context,
				user,
				Some(Options {
					title: None,
					error: "An owner is required.".into(),
				}),
			)
			.await
		}
	};
	let file = match file {
		Some(file) => file,
		None => {
			return render(
				&mut db,
				context,
				user,
				Some(Options {
					title: None,
					error: "A file is required.".into(),
				}),
			)
			.await
		}
	};

	// parse owner_id as user: id, or organization: id,
	let model = match tangram_core::types::Model::from_slice(&file) {
		Ok(model) => model,
		Err(_) => {
			return render(
				&mut db,
				context,
				user,
				Some(Options {
					title: Some(title),
					error: "Invalid Tangram model file.".into(),
				}),
			)
			.await;
		}
	};
	let now = Utc::now().timestamp();
	let repo_id = Id::new();

	let id_parts: Vec<&str> = dbg!(owner_id.split(':').collect());

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
			.bind(&title)
			.bind(&user_id.to_string())
			.execute(&mut *db)
			.await;
			if result.is_err() {
				return render(
					&mut db,
					context,
					user,
					Some(Options {
						title: Some(title),
						error: "A repo with this title already exists.".into(),
					}),
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
			.bind(&title)
			.bind(&organization_id.to_string())
			.execute(&mut *db)
			.await;
			if result.is_err() {
				return render(
					&mut db,
					context,
					user,
					Some(Options {
						title: Some(title),
						error: "A repo in this organization with this title already exists.".into(),
					}),
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
	.bind(&base64::encode(file))
	.execute(&mut *db)
	.await;

	if result.is_err() {
		return render(&mut db, context,
			user,
			Some(Options {
				title: Some(title),
				error: "This model has already been uploaded. Tangram models have unique identifiers and can only belong to one account.".into(),
			}),
		).await;
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
