use super::props::Props;
use tangram_app_common::Context;
use tangram_app_layouts::app_layout::get_app_layout_info;
use tangram_deps::{http, hyper, pinwheel::Pinwheel};
use tangram_util::error::Result;

pub async fn get(
	pinwheel: &Pinwheel,
	context: &Context,
	_request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	let app_layout_info = get_app_layout_info(context).await?;
	let props = Props {
		app_layout_info,
		error: None,
	};
	let html = pinwheel.render_with_props("/organizations/new", props)?;
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
