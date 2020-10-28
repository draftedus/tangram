use super::props::props;
use crate::Context;
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};

pub async fn get(
	context: &Context,
	request: Request<Body>,
	model_id: &str,
	column_name: &str,
) -> Result<Response<Body>> {
	let props = props(context, request, model_id, column_name).await?;
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/training_stats/columns/_column_name",
		props,
	)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
