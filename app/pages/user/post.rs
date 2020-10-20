use crate::common::user::User;
use crate::{
	common::{error::Error, user::authorize_user},
	Context,
};
use anyhow::Result;
use chrono::prelude::*;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "logout")]
	Logout,
}

pub async fn post(mut request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Err(Error::NotFound.into());
	}
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_user(&request, &mut db, context.options.auth_enabled)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let user = user.unwrap();
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let response = match action {
		Action::Logout => logout(user, &mut db).await?,
	};
	db.commit().await?;
	Ok(response)
}

async fn logout(user: User, db: &mut sqlx::Transaction<'_, sqlx::Any>) -> Result<Response<Body>> {
	let now = Utc::now().timestamp();
	sqlx::query(
		"
			update
				tokens
			set
				deleted_at = ?1
			where
				token = ?2
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
