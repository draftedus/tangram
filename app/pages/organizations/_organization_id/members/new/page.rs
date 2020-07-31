use crate::Context;
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

#[derive(serde::Serialize)]
struct Props {}

pub async fn page(
	_request: Request<Body>,
	context: &Context,
	_organization_id: &str,
) -> Result<Response<Body>> {
	let props = Props {};
	let html = context
		.pinwheel
		.render("/organizations/_organization_id/members/new", props)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}
