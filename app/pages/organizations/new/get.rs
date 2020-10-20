use super::props::Props;
use crate::Context;
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

pub async fn get(_request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let props = Props {};
	let html = context.pinwheel.render_with("/organizations/new", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
