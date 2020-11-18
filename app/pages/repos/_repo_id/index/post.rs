use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_deps::{http, hyper, pinwheel::Pinwheel, serde_urlencoded, sqlx};
use tangram_util::{error::Result, id::Id};

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
	_pinwheel: &Pinwheel,
	context: &Context,
	mut request: http::Request<hyper::Body>,
	repo_id: &str,
) -> Result<http::Response<hyper::Body>> {
	let data = match hyper::body::to_bytes(request.body_mut()).await {
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
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(http::header::LOCATION, format!("/repos/{}/", repo_id))
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}
