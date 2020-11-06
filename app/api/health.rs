use crate::Context;
use hyper::{Body, Request, Response, StatusCode};
use tangram_util::error::Result;

pub(crate) async fn get(context: &Context, _request: Request<Body>) -> Result<Response<Body>> {
	match context.pool.acquire().await {
		Ok(_) => Ok(Response::builder()
			.status(StatusCode::OK)
			.body(Body::empty())?),
		Err(_) => Ok(Response::builder()
			.status(StatusCode::SERVICE_UNAVAILABLE)
			.body(Body::empty())?),
	}
}
