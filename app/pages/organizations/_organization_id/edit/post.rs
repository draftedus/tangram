use crate::{
	common::{
		error::{bad_request, not_found, redirect_to_login, service_unavailable},
		user::{authorize_user, authorize_user_for_organization},
	},
	Context,
};
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use tangram_util::error::Result;
use tangram_util::id::Id;

#[derive(serde::Deserialize)]
struct Action {
	name: String,
}

pub async fn post(
	context: &Context,
	mut request: Request<Body>,
	organization_id: &str,
) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Ok(not_found());
	}
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
	let organization_id: Id = match organization_id.parse() {
		Ok(organization_id) => organization_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_organization(&mut db, &user, organization_id).await? {
		return Ok(not_found());
	};
	let Action { name } = action;
	sqlx::query(
		"
			update organizations
				set name = $1
			where organizations.id = $2
		",
	)
	.bind(&name)
	.bind(&organization_id.to_string())
	.execute(&mut *db)
	.await?;
	db.commit().await?;
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(
			header::LOCATION,
			format!("/organizations/{}/", organization_id),
		)
		.body(Body::empty())
		.unwrap();
	Ok(response)
}
