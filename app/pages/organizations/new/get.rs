use super::props::props;
use crate::Context;
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

pub async fn get(context: &Context, _request: Request<Body>) -> Result<Response<Body>> {
	let props = props(context, None).await?;
	let html = context.pinwheel.render_with("/organizations/new", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
