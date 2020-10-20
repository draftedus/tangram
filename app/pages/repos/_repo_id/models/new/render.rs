use super::props::Props;
use crate::Context;
use anyhow::Result;
use hyper::{Body, Response, StatusCode};

pub struct Options {
	pub error: String,
}

pub async fn render(context: &Context, options: Option<Options>) -> Result<Response<Body>> {
	let props = Props {
		error: options.as_ref().map(|o| o.error.to_owned()),
	};
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/new", props)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
