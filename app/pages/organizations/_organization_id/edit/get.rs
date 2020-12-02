use super::page::{render, Props};
use tangram_app_common::{error::not_found, Context};
use tangram_app_layouts::{app_layout::get_app_layout_info, document::PageInfo};
use tangram_deps::{http, hyper};
use tangram_util::error::Result;

pub async fn get(
	context: &Context,
	_request: http::Request<hyper::Body>,
	_organization_id: &str,
) -> Result<http::Response<hyper::Body>> {
	if !context.options.auth_enabled {
		return Ok(not_found());
	}
	let app_layout_info = get_app_layout_info(context).await?;
	let props = Props {
		app_layout_info,
		error: None,
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
