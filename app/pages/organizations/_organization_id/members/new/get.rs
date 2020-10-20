use super::props::Props;
use crate::{common::error::Error, Context};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

pub async fn get(
	_request: Request<Body>,
	context: &Context,
	_organization_id: &str,
) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Err(Error::NotFound.into());
	}
	let props = Props {};
	let html = context
		.pinwheel
		.render_with("/organizations/_organization_id/members/new", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
