use crate::{
	error::Error,
	user::{authorize_user, User},
	Context,
};
use anyhow::Result;
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action", rename_all = "camelCase")]
enum Action {
	Logout,
}

pub async fn post(mut request: Request<Body>, context: &Context) -> Result<Response<Body>> {
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
		Action::Logout => logout(user, db).await,
	}
}

pub async fn logout(user: User, db: deadpool_postgres::Transaction<'_>) -> Result<Response<Body>> {
	db.execute(
		"
			update
				tokens
			set
				deleted_at = now()
			where
				token = $1
		",
		&[&user.token],
	)
	.await?;
	db.commit().await?;
	Ok(Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/login")
		.header(header::SET_COOKIE, "auth=; Path=/; Max-Age=0")
		.body(Body::empty())?)
}
