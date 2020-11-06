use crate::{
	common::{
		error::{bad_request, not_found, redirect_to_login, service_unavailable},
		user::{authorize_user, authorize_user_for_model},
	},
	Context,
};
use hyper::{header, Body, Request, Response, StatusCode};
use sqlx::prelude::*;
use tangram_util::error::Result;
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
	context: &Context,
	request: Request<Body>,
	model_id: &str,
) -> Result<Response<Body>> {
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
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
			models.id = $1
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
	context: &Context,
	request: Request<Body>,
	model_id: &str,
) -> Result<Response<Body>> {
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let row = sqlx::query(
		"
		select
			data
		from models
		where
			models.id = $1
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
