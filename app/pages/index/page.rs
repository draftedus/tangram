use crate::{error::Error, user::authorize_user, Context};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

pub async fn page(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let mut db = context
		.database_pool
		.get()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let db = db.transaction().await?;
	let user = authorize_user(&request, &db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let props = super::data::data(&db, &user).await?;
	db.commit().await?;
	let html = context.pinwheel.render("/", props).await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}
