use crate::{
	common::{
		error::{bad_request, not_found, redirect_to_login, service_unavailable},
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
	let data = match to_bytes(request.body_mut()).await {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let action: Action = match serde_urlencoded::from_bytes(&data) {
		Ok(action) => action,
		Err(_) => return Ok(bad_request()),
	};
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	match action {
		Action::DeleteModel(DeleteModelAction { model_id, .. }) => {
			let model_id: Id = match model_id.parse() {
				Ok(model_id) => model_id,
				Err(_) => return Ok(bad_request()),
			};
			if !authorize_user_for_model(&mut db, &user, model_id).await? {
				return Ok(not_found());
			};
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
