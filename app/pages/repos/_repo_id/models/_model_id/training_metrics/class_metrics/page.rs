use crate::app::Context;
use anyhow::Result;
use html::html;
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;
use std::sync::Arc;

pub async fn page(
	_request: Request<Body>,
	_context: Arc<Context>,
	_model_id: &str,
	_search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let html = html!(<div>"Hello World"</div>);
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html.render_to_string()))
		.unwrap())
}
