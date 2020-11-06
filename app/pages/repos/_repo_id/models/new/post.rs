use super::render::render;
use crate::{
	common::{
		error::{not_found, redirect_to_login, service_unavailable},
		user::{authorize_user, authorize_user_for_repo},
	},
	Context,
};
use bytes::Buf;
use chrono::prelude::*;
use hyper::{header, Body, Request, Response, StatusCode};
use multer::Multipart;
use tangram_util::error::Result;
use tangram_util::id::Id;

pub async fn post(
	context: &Context,
	request: Request<Body>,
	repo_id: &str,
) -> Result<Response<Body>> {
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let repo_id: Id = match repo_id.parse() {
		Ok(repo_id) => repo_id,
		Err(_) => return Ok(not_found()),
	};
	if !authorize_user_for_repo(&mut db, &user, repo_id).await? {
		return Ok(not_found());
	}
	let boundary = match request
		.headers()
		.get(header::CONTENT_TYPE)
		.and_then(|ct| ct.to_str().ok())
		.and_then(|ct| multer::parse_boundary(ct).ok())
	{
		Some(boundary) => boundary,
		None => {
			let html = render(context, Some("Failed to parse request body.".to_owned())).await?;
			let response = Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(Body::from(html))
				.unwrap();
			return Ok(response);
		}
	};
	let mut file: Option<Vec<u8>> = None;
	let mut multipart = Multipart::new(request.into_body(), boundary);
	while let Some(mut field) = multipart.next_field().await? {
		let name = match field.name() {
			Some(name) => name.to_owned(),
			None => {
				let html =
					render(context, Some("Failed to parse request body.".to_owned())).await?;
				let response = Response::builder()
					.status(StatusCode::BAD_REQUEST)
					.body(Body::from(html))
					.unwrap();
				return Ok(response);
			}
		};
		let mut field_data = Vec::new();
		while let Some(chunk) = field.chunk().await? {
			field_data.extend(chunk.bytes());
		}
		match name.as_str() {
			"file" => file = Some(field_data),
			_ => {
				let html =
					render(context, Some("Failed to parse request body.".to_owned())).await?;
				let response = Response::builder()
					.status(StatusCode::BAD_REQUEST)
					.body(Body::from(html))
					.unwrap();
				return Ok(response);
			}
		}
	}
	let file = match file {
		Some(file) => file,
		None => {
			let html = render(context, Some("A file is required.".to_owned())).await?;
			let response = Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(Body::from(html))
				.unwrap();
			return Ok(response);
		}
	};
	let model = match tangram_core::model::Model::from_slice(&file) {
		Ok(model) => model,
		Err(_) => {
			let html = render(context, Some("Invalid tangram model file.".to_owned())).await?;
			let response = Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(Body::from(html))
				.unwrap();
			return Ok(response);
		}
	};
	let now = Utc::now().timestamp();
	let result = sqlx::query(
		"
			insert into models
				(id, repo_id, created_at, data)
			values
				($1, $2, $3, $4)
		",
	)
	.bind(&model.id().to_string())
	.bind(&repo_id.to_string())
	.bind(&now)
	.bind(&base64::encode(file))
	.execute(&mut *db)
	.await;
	if result.is_err() {
		let html = render(
			context,
			Some("There was an error uploading your model.".to_owned()),
		)
		.await?;
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
