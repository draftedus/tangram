use crate::Context;
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

#[derive(serde::Serialize)]
struct Props {}

pub async fn get(_request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	let props = Props {};
	let html = context
		.pinwheel
		.render("/repos/_repo_id/models/_model_id/new", props)
		.await?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}
