use super::props::Props;
use crate::{common::error::not_found, Context};
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;
use tangram_util::error::Result;

pub async fn get(
	context: &Context,
	_request: Request<Body>,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Ok(not_found());
	}
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
