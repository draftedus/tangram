use crate::{
	common::{
		error::Error,
		user::{authorize_normal_user, NormalUser},
	},
	Context,
};
use chrono::prelude::*;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
use tangram_util::error::Result;

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "logout")]
	Logout,
}

pub async fn post(context: &Context, mut request: Request<Body>) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Err(Error::NotFound.into());
	}
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_normal_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let response = match action {
		Action::Logout => logout(&user, &mut db).await?,
	};
	db.commit().await?;
	Ok(response)
}

async fn logout(
	user: &NormalUser,
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Response<Body>> {
	let now = Utc::now().timestamp();
	sqlx::query(
		"
			update
				tokens
			set
				deleted_at = $1
			where
				token = $2
		",
	)
	.bind(&now)
	.bind(&user.token)
	.execute(&mut *db)
	.await?;
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/login")
		.header(header::SET_COOKIE, "auth=; Path=/; Max-Age=0")
		.body(Body::empty())
		.unwrap();
	Ok(response)
}
