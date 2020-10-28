use super::props::props;
use crate::{common::error::Error, Context};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

pub async fn get(
	context: &Context,
	_request: Request<Body>,
	_organization_id: &str,
) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Err(Error::NotFound.into());
	}
	let props = props(context, None).await?;
	let html = context
		.pinwheel
		.render_with("/organizations/_organization_id/edit", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
