pub mod models;

use crate::Context;
use anyhow::Result;
use html::html;
use hyper::{Body, Request, Response, StatusCode};

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelListResponse {
	pub models: Vec<ModelListItem>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelListItem {
	pub id: String,
	pub title: String,
	pub created_at: String,
	pub organization_id: String,
	pub organization_name: String,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelResponse {
	pub id: String,
	pub title: String,
	pub created_at: String,
	pub data: String,
	pub owner_name: String,
}

pub async fn get(_request: Request<Body>, _context: &Context) -> Result<Response<Body>> {
	let html = html!(<div>"Hello World"</div>);
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html.render_to_string()))
		.unwrap())
}
