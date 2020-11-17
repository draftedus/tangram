use tangram_app_common::Context;
use tangram_deps::{http, hyper};
use tangram_util::error::Result;

pub async fn get(
	context: &Context,
	_request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	match context.pool.acquire().await {
		Ok(_) => Ok(http::Response::builder()
			.status(http::StatusCode::OK)
			.body(hyper::Body::empty())?),
		Err(_) => Ok(http::Response::builder()
			.status(http::StatusCode::SERVICE_UNAVAILABLE)
			.body(hyper::Body::empty())?),
	}
}
