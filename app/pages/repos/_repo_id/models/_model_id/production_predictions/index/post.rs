use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	http, hyper, serde_urlencoded,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_util::{error::Result, id::Id};

#[derive(serde::Deserialize)]
struct Action {
	identifier: String,
}

pub async fn post(
	context: &Context,
	mut request: http::Request<hyper::Body>,
	model_id: &str,
) -> Result<http::Response<hyper::Body>> {
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
	let data = match hyper::body::to_bytes(request.body_mut()).await {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let Action { identifier } = match serde_urlencoded::from_bytes(&data) {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	// Redirect.
	let path = format!("predictions/{}", identifier);
	Ok(http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(http::header::LOCATION, path)
		.body(hyper::Body::empty())?)
}
