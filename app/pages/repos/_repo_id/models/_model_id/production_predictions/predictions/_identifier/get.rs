use super::props::props;
use crate::Context;
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

pub async fn get(
	context: &Context,
	request: Request<Body>,
	model_id: &str,
	identifier: &str,
) -> Result<Response<Body>> {
	let props = props(context, request, model_id, identifier).await?;
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/production_predictions/predictions/_identifier",
		props,
	)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
