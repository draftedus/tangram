#![allow(non_snake_case)]

use anyhow::Result;
use futures::FutureExt;
use hyper::{service::service_fn, Body, Request, Response, StatusCode};
use pinwheel::Pinwheel;
use std::{panic::AssertUnwindSafe, path::PathBuf, sync::Arc};

pub struct Context {
	pinwheel: Pinwheel,
}

fn content_type(path: &str) -> Option<&'static str> {
	if path.ends_with(".js") {
		Some("text/javascript")
	} else if path.ends_with(".svg") {
		Some("image/svg+xml")
	} else {
		None
	}
}

async fn handle(
	request: Request<Body>,
	context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
	let method = request.method().clone();
	let uri = request.uri().clone();
	let path_and_query = uri.path_and_query().unwrap();
	let path = path_and_query.path();
	// serve static files from pinwheel
	if let Some(data) = context.pinwheel.serve(path) {
		let mut response = Response::builder();
		if let Some(content_type) = content_type(path) {
			response = response.header("content-type", content_type);
		}
		let response = response.body(Body::from(data)).unwrap();
		return Ok(response);
	}
	let html = context
		.pinwheel
		.render(path, serde_json::Value::Null)
		.await
		.unwrap();
	let response = Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap();
	eprintln!("{} {} {}", method, path, response.status().as_u16());
	Ok(response)
}

#[tokio::main]
pub async fn main() -> Result<()> {
	// get host and port
	let host = std::env::var("HOST")
		.map(|host| host.parse().expect("HOST environment variable invalid"))
		.unwrap_or_else(|_| "0.0.0.0".parse().unwrap());
	let port = std::env::var("PORT")
		.map(|port| port.parse().expect("PORT environment variable invalid"))
		.unwrap_or_else(|_| 80);
	let addr = std::net::SocketAddr::new(host, port);

	let pinwheel = Pinwheel::dev(PathBuf::from("."), PathBuf::from("dist"));

	// create the context
	let context = Arc::new(Context { pinwheel });

	// start the server
	let listener = std::net::TcpListener::bind(&addr).unwrap();
	let mut listener = tokio::net::TcpListener::from_std(listener).unwrap();
	let http = hyper::server::conn::Http::new();
	eprintln!("🚀 serving on port {}", port);

	// wait for and handle each connection
	loop {
		let result = listener.accept().await;
		let (socket, _) = match result {
			Ok(s) => s,
			Err(e) => {
				eprintln!("tcp error: {}", e);
				continue;
			}
		};
		let context = context.clone();
		tokio::spawn(
			http.serve_connection(
				socket,
				service_fn(move |request| {
					let context = context.clone();
					AssertUnwindSafe(handle(request, context))
						.catch_unwind()
						.map(|result| match result {
							Err(_) => Ok(Response::builder()
								.status(StatusCode::INTERNAL_SERVER_ERROR)
								.body(Body::from("internal server error"))
								.unwrap()),
							Ok(response) => response,
						})
				}),
			)
			.map(|r| {
				if let Err(e) = r {
					eprintln!("http error: {}", e);
				}
			}),
		);
	}
}
