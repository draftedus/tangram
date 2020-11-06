use super::props::Props;
use crate::{common::error::not_found, Context};
use std::collections::BTreeMap;
use tangram_util::error::Result;

pub async fn get(
	context: &Context,
	_request: http::Request<hyper::Body>,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<http::Response<hyper::Body>> {
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
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
