use super::props::props;
use crate::{
	common::{error::Error, user::authorize_user},
	Context,
};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

pub async fn get(request: Request<Body>, context: &Context) -> Result<Response<Body>> {
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
	let props = props(&mut db, user).await?;
	db.commit().await?;
	let html = context.pinwheel.render_with("/user", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
