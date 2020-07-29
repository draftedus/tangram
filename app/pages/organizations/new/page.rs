use crate::app::Context;
use anyhow::Result;
use html::html;
use hyper::{Body, Request, Response, StatusCode};

pub async fn page(_request: Request<Body>, _context: &Context) -> Result<Response<Body>> {
	let html = html!(<div>"Hello World"</div>);
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html.render_to_string()))
		.unwrap())
}
