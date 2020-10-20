use super::props::props;
use crate::{common::error::Error, Context};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

pub async fn get(
	request: Request<Body>,
	context: &Context,
	organization_id: &str,
) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Err(Error::NotFound.into());
	}
	let props = props(request, context, organization_id).await?;
	let html = context
		.pinwheel
		.render_with("/organizations/_organization_id/", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
