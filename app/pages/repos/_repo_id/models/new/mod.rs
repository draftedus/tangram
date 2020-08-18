use crate::{
	error::Error,
	user::{authorize_user, authorize_user_for_repo},
	Context,
};
use anyhow::Result;
use bytes::Buf;
use chrono::prelude::*;
use hyper::{header, Body, Request, Response, StatusCode};
use multer::Multipart;
use tangram_core::id::Id;

#[derive(serde::Serialize)]
struct Props {
	error: Option<String>,
}

pub async fn get(_request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let props = Props { error: None };
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
	let user = authorize_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let repo_id: Id = repo_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_repo(&mut db, &user, repo_id).await? {
		return Err(Error::Unauthorized.into());
	}
	let boundary = request
		.headers()
		.get(header::CONTENT_TYPE)
		.and_then(|ct| ct.to_str().ok())
		.and_then(|ct| multer::parse_boundary(ct).ok());
	let boundary = boundary.ok_or_else(|| Error::BadRequest)?;
	let mut file: Option<Vec<u8>> = None;
	let mut multipart = Multipart::new(request.into_body(), boundary);
	while let Some(mut field) = multipart.next_field().await? {
		let name = field.name().ok_or_else(|| Error::BadRequest)?.to_owned();
		let mut field_data = Vec::new();
		while let Some(chunk) = field.chunk().await? {
			field_data.extend(chunk.bytes());
		}
		match name.as_str() {
			"file" => file = Some(field_data),
			_ => return Err(Error::BadRequest.into()),
		};
	}
	let file = file.ok_or_else(|| Error::BadRequest)?;
	let model = match tangram_core::types::Model::from_slice(&file) {
		Ok(model) => model,
		Err(_) => {
			let props = Props {
				error: Some("invalid tangram model".into()),
			};
			let html = context
				.pinwheel
				.render_with("/repos/_repo_id/models/new", props)?;
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
		let props = Props {
			error: Some("model has already been uploaded".into()),
		};
		let html = context
			.pinwheel
			.render_with("/repos/_repo_id/models/new", props)?;
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
			format!("/repos/{}/models/{}/", repo_id, model.id().to_string()),
		)
		.body(Body::empty())
		.unwrap();
	Ok(response)
}
