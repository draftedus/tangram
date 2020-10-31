use crate::{Context, Error};
use hyper::{Body, Request, Response, StatusCode};
use tangram_util::error::Result;

pub(crate) async fn get(context: &Context, _request: Request<Body>) -> Result<Response<Body>> {
	match context.pool.acquire().await {
		Ok(_) => {
			let response = Response::builder()
				.status(StatusCode::OK)
				.body(Body::empty())
				.unwrap();
			Ok(response)
		}
		Err(_) => Err(Error::ServiceUnavailable.into()),
	}
}
