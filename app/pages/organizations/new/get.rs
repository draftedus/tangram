use super::props::Props;
use tangram_app_common::{http, hyper, Context};
use tangram_app_layouts::app_layout::get_app_layout_info;
use tangram_util::error::Result;

pub async fn get(
	context: &Context,
	_request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	let app_layout_info = get_app_layout_info(context).await?;
	let props = Props {
		app_layout_info,
		error: None,
	};
	let html = context.pinwheel.render_with("/organizations/new", props)?;
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
