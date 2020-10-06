use crate::{
	common::{
		error::Error,
		user::{authorize_user, authorize_user_for_repo},
	},
	Context,
};
use anyhow::Result;
use bytes::Buf;
use chrono::prelude::*;
use hyper::{header, Body, Request, Response, StatusCode};
use multer::Multipart;
use tangram_id::Id;

#[derive(serde::Serialize)]
struct Props {
	error: Option<String>,
}

struct Options {
	error: String,
}

pub async fn get(
	request: Request<Body>,
	context: &Context,
	repo_id: &str,
) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let repo_id: Id = repo_id.parse().map_err(|_| Error::NotFound)?;
	if let Some(user) = user {
		if !authorize_user_for_repo(&mut db, &user, repo_id).await? {
			return Err(Error::NotFound.into());
		}
	}
	let response = render(context, None).await;
	db.commit().await?;
	response
}

async fn render(context: &Context, options: Option<Options>) -> Result<Response<Body>> {
	let props = Props {
		error: options.as_ref().map(|o| o.error.to_owned()),
	};
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/new", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

pub async fn post(
	request: Request<Body>,
	context: &Context,
	repo_id: &str,
) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let repo_id: Id = repo_id.parse().map_err(|_| Error::NotFound)?;
	if let Some(user) = user {
		if !authorize_user_for_repo(&mut db, &user, repo_id).await? {
			return Err(Error::Unauthorized.into());
		}
	}
	let boundary = match request
		.headers()
		.get(header::CONTENT_TYPE)
		.and_then(|ct| ct.to_str().ok())
		.and_then(|ct| multer::parse_boundary(ct).ok())
	{
		Some(boundary) => boundary,
		None => {
			return render(
				context,
				Some(Options {
					error: "Failed to parse request body.".into(),
				}),
			)
			.await
		}
	};
	let mut file: Option<Vec<u8>> = None;
	let mut multipart = Multipart::new(request.into_body(), boundary);
	while let Some(mut field) = multipart.next_field().await? {
		let name = match field.name() {
			Some(name) => name.to_owned(),
			None => {
				return render(
					context,
					Some(Options {
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
			"file" => file = Some(field_data),
			_ => {
				return render(
					context,
					Some(Options {
						error: "Failed to parse request body.".into(),
					}),
				)
				.await
			}
		}
	}
	let file = match file {
		Some(file) => file,
		None => {
			return render(
				context,
				Some(Options {
					error: "A file is required.".into(),
				}),
			)
			.await
		}
	};
	let model = match tangram_core::model::Model::from_slice(&file) {
		Ok(model) => model,
		Err(_) => {
			return render(
				context,
				Some(Options {
					error: "Invalid tangram model file.".into(),
				}),
			)
			.await
		}
	};
	let now = Utc::now().timestamp();
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
		return render(
			context,
			Some(Options {
				error: "There was an error uploading your model.".into(),
			}),
		)
		.await;
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
