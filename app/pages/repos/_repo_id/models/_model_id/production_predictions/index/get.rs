use super::props::props;
use crate::Context;
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let after = search_params
		.as_ref()
		.and_then(|s| s.get("after"))
		.and_then(|t| t.parse().ok());
	let before = search_params
		.as_ref()
		.and_then(|s| s.get("before"))
		.and_then(|t| t.parse().ok());
	let props = props(request, context, model_id, before, after).await?;
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/production_predictions/",
		props,
	)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
