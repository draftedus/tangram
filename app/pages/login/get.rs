use crate::page::render;
use std::collections::BTreeMap;
use tangram_app_common::{error::not_found, Context};
use tangram_app_layouts::document::PageInfo;
use tangram_deps::{http, hyper};
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
	let props = crate::page::Props {
		code: email.is_some(),
		error: None,
		email,
	};
	let page_info = PageInfo {
		client_wasm_js_src: None,
	};
	let html = render(props, page_info);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
