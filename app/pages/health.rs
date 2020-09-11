use crate::app::{Context, Error};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

pub async fn get(_request: Request<Body>, context: &Context) -> Result<Response<Body>> {
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
