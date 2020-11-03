use crate::{
	common::{
		error::Error,
		user::{authorize_user, authorize_user_for_model},
	},
	Context,
};
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use tangram_util::error::Result;
use tangram_util::id::Id;

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete_model")]
	DeleteModel(DeleteModelAction),
}

#[derive(serde::Deserialize)]
struct DeleteModelAction {
	model_id: String,
}

pub async fn post(
	context: &Context,
	mut request: Request<Body>,
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
			authorize_user_for_model(&mut db, &user, model_id)
				.await
				.map_err(|_| Error::NotFound)?;
			sqlx::query(
				"
					delete from models
					where id = $1
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
