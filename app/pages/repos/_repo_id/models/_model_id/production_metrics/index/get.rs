use super::props::props;
use crate::{common::date_window::get_date_window_and_interval, Context};
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use std::collections::BTreeMap;

pub async fn get(
	request: Request<Body>,
	context: &Context,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let (date_window, date_window_interval) = get_date_window_and_interval(&search_params)?;
	let props = props(
		request,
		context,
		model_id,
		date_window,
		date_window_interval,
	)
	.await?;
	let html = context.pinwheel.render_with(
		"/repos/_repo_id/models/_model_id/production_metrics/",
		props,
	)?;
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	Ok(response)
}
