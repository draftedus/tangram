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
struct Props {}

pub async fn get(_request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let props = Props {};
	let html = context
		.pinwheel
		.render("/repos/_repo_id/models/_model_id/new", props)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

pub async fn post(
	request: Request<Body>,
	context: &Context,
	repo_id: &str,
) -> Result<Response<Body>> {
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let repo_id: Id = repo_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_repo(&db, &user, repo_id).await? {
		return Err(Error::Unauthorized.into());
	}
	let boundary = request
		.headers()
		.get(header::CONTENT_TYPE)
		.and_then(|ct| ct.to_str().ok())
		.and_then(|ct| multer::parse_boundary(ct).ok());
	let boundary = boundary.ok_or_else(|| Error::BadRequest)?;
	let mut title: Option<String> = None;
	let mut data: Option<Vec<u8>> = None;
	let mut multipart = Multipart::new(request.into_body(), boundary);
	while let Some(mut field) = multipart.next_field().await? {
		let name = field.name().ok_or_else(|| Error::BadRequest)?.to_owned();
		let mut field_data = Vec::new();
		while let Some(chunk) = field.chunk().await? {
			field_data.extend(chunk.bytes());
		}
		match name.as_str() {
			"title" => title = Some(String::from_utf8(field_data).map_err(|_| Error::BadRequest)?),
			"model" => data = Some(field_data),
			_ => return Err(Error::BadRequest.into()),
		};
	}
	let title = title.ok_or_else(|| Error::BadRequest)?;
	let data = data.ok_or_else(|| Error::BadRequest)?;
	let model = tangram_core::types::Model::from_slice(&data).map_err(|_| Error::BadRequest)?;
	let created_at: DateTime<Utc> = Utc::now();
	db.execute(
		"
			insert into models
				(id, repo_id, title, created_at, data, is_main)
			values
				($1, $2, $3, $4, $5, $6)
		",
		&[&model.id(), &repo_id, &title, &created_at, &data, &false],
	)
	.await?;
	db.commit().await?;
	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, format!("/repos/{}", repo_id))
		.body(Body::empty())?)
}
