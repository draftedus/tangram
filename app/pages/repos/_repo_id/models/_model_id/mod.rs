use crate::{
	common::{
		error::Error,
		user::{authorize_user, authorize_user_for_model},
	},
	Context,
};
use anyhow::Result;
use hyper::{header, Body, Request, Response, StatusCode};
use sqlx::prelude::*;
use tangram_util::id::Id;

pub mod index;
pub mod prediction;
pub mod production_metrics;
pub mod production_predictions;
pub mod production_stats;
pub mod training_importances;
pub mod training_metrics;
pub mod training_stats;
pub mod tuning;

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete_model")]
	DeleteModel,
	#[serde(rename = "download_model")]
	DownloadModel,
}

pub async fn post(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if let Some(user) = user {
		if !authorize_user_for_model(&mut db, &user, model_id).await? {
			return Err(Error::NotFound.into());
		}
	}
	let response = delete_model(&mut db, model_id).await?;
	db.commit().await?;
	Ok(response)
}

async fn delete_model(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
) -> Result<Response<Body>> {
	sqlx::query(
		"
		delete from models
		where
			models.id = ?1
	",
	)
	.bind(&model_id.to_string())
	.execute(&mut *db)
	.await?;
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/")
		.body(Body::empty())
		.unwrap();
	Ok(response)
}

pub async fn download(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
) -> Result<Response<Body>> {
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if let Some(user) = user {
		if !authorize_user_for_model(&mut db, &user, model_id).await? {
			return Err(Error::NotFound.into());
		}
	}
	let row = sqlx::query(
		"
		select
			data
		from models
		where
			models.id = ?1
		",
	)
	.bind(&model_id.to_string())
	.fetch_one(&mut *db)
	.await?;
	let data: String = row.get(0);
	let data = base64::decode(data)?;
	db.commit().await?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(data))
		.unwrap();
	Ok(response)
}
