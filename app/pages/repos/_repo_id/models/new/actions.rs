use crate::app::{
	error::Error,
	user::{authorize_user, authorize_user_for_repo, User},
	Context,
};
use anyhow::Result;
use chrono::prelude::*;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use tangram::id::Id;

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "upload_model_version")]
	UploadModelVersion(UploadModelVersionAction),
}

#[derive(serde::Deserialize, Debug)]
struct UploadModelVersionAction {
	pub title: String,
	pub data: String,
	#[serde(rename = "repoId")]
	pub repo_id: String,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct UploadModelVersionResponse {
	pub id: String,
	pub title: String,
	#[serde(rename = "createdAt")]
	pub created_at: String,
	pub data: String,
}

pub async fn actions(mut request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	match action {
		Action::UploadModelVersion(action) => upload_model_version(action, user, db).await,
	}
}

async fn upload_model_version(
	action: UploadModelVersionAction,
	user: User,
	db: deadpool_postgres::Transaction<'_>,
) -> Result<Response<Body>> {
	let UploadModelVersionAction {
		repo_id,
		data,
		title,
	} = action;

	let repo_id: Id = repo_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_user_for_repo(&db, &user, repo_id).await? {
		return Err(Error::Unauthorized.into());
	}
	let model_data: Vec<u8> = base64::decode(&data).map_err(|_| Error::BadRequest)?;
	let model = tangram::types::Model::from_slice(&model_data).map_err(|_| Error::BadRequest)?;
	let created_at: DateTime<Utc> = Utc::now();
	db.execute(
		"
			insert into models
				(id, repo_id, title, created_at, data, is_main)
			values
				($1, $2, $3, $4, $5, $6)
		",
		&[
			&model.id(),
			&repo_id,
			&title,
			&created_at,
			&model_data,
			&false,
		],
	)
	.await?;
	db.commit().await?;
	let response = UploadModelVersionResponse {
		id: model.id().to_string(),
		title,
		data,
		created_at: created_at.to_rfc3339(),
	};
	let response = serde_json::to_vec(&response)?;

	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, format!("/repos/{}", repo_id))
		.body(Body::from(response))?)
}
