use super::props::Props;
use crate::Context;
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;

pub async fn get(
	context: &Context,
	_request: Request<Body>,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let email = search_params.as_ref().and_then(|s| s.get("email").cloned());
	let props = Props {
		code: email.is_some(),
		error: None,
		email,
	};
	let html = context.pinwheel.render_with("/login", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
