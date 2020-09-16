use crate::{
	common::user::{authorize_user, authorize_user_for_model, authorize_user_for_repo},
	error::Error,
	Context,
};
use anyhow::Result;
use chrono::prelude::*;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use sqlx::prelude::*;
use tangram_id::Id;

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
		authorize_user_for_repo(&mut db, &user, repo_id)
			.await
			.map_err(|_| Error::NotFound)?;
	}
	let props = props(&mut db, repo_id).await?;
	db.commit().await?;
	let html = context.pinwheel.render_with("/repos/_repo_id/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub models: Vec<Model>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Model {
	pub id: String,
	pub created_at: String,
}

pub async fn props(db: &mut sqlx::Transaction<'_, sqlx::Any>, repo_id: Id) -> Result<Props> {
	let rows = sqlx::query(
		"
			select
				models.id,
				models.created_at
			from models
			where models.repo_id = ?1
			order by models.created_at
		",
	)
	.bind(&repo_id.to_string())
	.fetch_all(db)
	.await?;
	let models = rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let id: Id = id.parse().unwrap();
			let created_at: i64 = row.get(1);
			let created_at: DateTime<Utc> = Utc.timestamp(created_at, 0);
			Model {
				created_at: created_at.to_rfc3339(),
				id: id.to_string(),
			}
		})
		.collect();
	Ok(Props { models })
}

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete_model")]
	DeleteModel(DeleteModelAction),
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteModelAction {
	pub model_id: String,
}

pub async fn post(
	mut request: Request<Body>,
	context: &Context,
	repo_id: &str,
) -> Result<Response<Body>> {
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;

	match action {
		Action::DeleteModel(DeleteModelAction { model_id, .. }) => {
			let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
			if let Some(user) = user {
				authorize_user_for_model(&mut db, &user, model_id)
					.await
					.map_err(|_| Error::NotFound)?;
			}
			sqlx::query(
				"
					delete from models
					where id = ?1
				",
			)
			.bind(&model_id.to_string())
			.execute(&mut *db)
			.await?;
		}
	}

	db.commit().await?;

	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, format!("/repos/{}/", repo_id))
		.body(Body::empty())
		.unwrap();
	Ok(response)
}
