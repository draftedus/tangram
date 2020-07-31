use crate::{
	error::Error,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use anyhow::Result;
use hyper::{header, Body, Request, Response, StatusCode};
use tangram_core::id::Id;
use tokio_postgres as postgres;

pub mod index;
pub mod introspect;
pub mod predict;
pub mod production_metrics;
pub mod production_stats;
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
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	let response = delete_model(&db, model_id).await?;
	db.commit().await?;
	Ok(response)
}

async fn delete_model(db: &postgres::Transaction<'_>, model_id: Id) -> Result<Response<Body>> {
	db.execute(
		"
			delete from models
			where
				models.id = $1
		",
		&[&model_id],
	)
	.await?;
	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/")
		.body(Body::empty())?)
}

pub async fn download(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
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
	let model_id: Id = model_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_model(&db, &user, model_id).await? {
		return Err(Error::NotFound.into());
	}
	let row = db
		.query_one(
			"
				select
					data
				from models
				where
					models.id = $1
			",
			&[&model_id],
		)
		.await?;
	db.commit().await?;
	let data: Vec<u8> = row.get(0);
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(data))
		.unwrap())
}
