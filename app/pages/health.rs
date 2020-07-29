use crate::app::{Context, Error};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

pub async fn get(_request: Request<Body>, context: &Context) -> Result<Response<Body>> {
	match context.database_pool.get().await {
		Ok(_) => Ok(Response::builder()
			.status(StatusCode::OK)
			.body(Body::empty())?),
		Err(_) => Err(Error::ServiceUnavailable.into()),
	}
}
