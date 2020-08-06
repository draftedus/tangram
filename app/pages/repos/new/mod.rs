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
	let props = props(&mut db, user).await?;
	db.commit().await?;
	let html = context.pinwheel.render("/repos/new", props).await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Props {
	organizations: Vec<Organization>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Organization {
	id: String,
	name: String,
}

async fn props(db: &mut sqlx::Transaction<'_, sqlx::Any>, user: User) -> Result<Props> {
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
	let items: Vec<_> = rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let name: String = row.get(1);
			Organization { id, name }
		})
		.collect();
	Ok(Props {
		organizations: items,
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
	let boundary = request
		.headers()
		.get(header::CONTENT_TYPE)
		.and_then(|ct| ct.to_str().ok())
		.and_then(|ct| multer::parse_boundary(ct).ok())
		.ok_or_else(|| Error::BadRequest)?;
	let mut title: Option<String> = None;
	let mut organization_id: Option<String> = None;
	let mut file: Option<Vec<u8>> = None;
	let mut multipart = Multipart::new(request.into_body(), boundary);
	while let Some(mut field) = multipart.next_field().await? {
		let name = field.name().ok_or_else(|| Error::BadRequest)?.to_owned();
		let mut field_data = Vec::new();
		while let Some(chunk) = field.chunk().await? {
			field_data.extend(chunk.bytes());
		}
		match name.as_str() {
			"title" => title = Some(String::from_utf8(field_data).map_err(|_| Error::BadRequest)?),
			"organization_id" => {
				organization_id =
					Some(String::from_utf8(field_data).map_err(|_| Error::BadRequest)?)
			}
			"file" => file = Some(field_data),
			_ => return Err(Error::BadRequest.into()),
		};
	}
	let title = title.ok_or_else(|| Error::BadRequest)?;
	let organization_id: Id = organization_id
		.ok_or(Error::BadRequest)?
		.parse()
		.map_err(|_| Error::BadRequest)?;
	let file = file.ok_or_else(|| Error::BadRequest)?;
	if !authorize_user_for_organization(&mut db, &user, organization_id).await? {
		return Err(Error::Unauthorized.into());
	}
	let model = tangram_core::types::Model::from_slice(&file).map_err(|_| Error::BadRequest)?;
	let now = Utc::now().timestamp();
	let repo_id = Id::new();
	sqlx::query(
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
	.await?;
	sqlx::query(
		"
			insert into models
				(id, repo_id, title, created_at, data, is_main)
			values
				(?1, ?2, ?3, ?4, ?5, ?6)
		",
	)
	.bind(&model.id().to_string())
	.bind(&repo_id.to_string())
	.bind(&title)
	.bind(&now)
	.bind(&base64::encode(file))
	.bind(&true)
	.execute(&mut *db)
	.await?;
	db.commit().await?;
	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(
			header::LOCATION,
			format!("/repos/{}/models/{}/", repo_id, model.id().to_string()),
		)
		.body(Body::empty())?)
}
