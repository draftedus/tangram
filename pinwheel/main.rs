#![allow(non_snake_case)]

use anyhow::Result;
use futures::FutureExt;
use hyper::{service::service_fn, Body, Response, StatusCode};
use pinwheel::Pinwheel;
use std::{
	panic::AssertUnwindSafe,
	path::{Path, PathBuf},
	sync::Arc,
};

#[tokio::main]
pub async fn main() -> Result<()> {
	let mut app = clap::App::new("pinwheel")
		.version(clap::crate_version!())
		.setting(clap::AppSettings::SubcommandRequiredElseHelp);
	app = app.subcommand(clap::SubCommand::with_name("dev").about("run the development server"));
	app = app.subcommand(
		clap::SubCommand::with_name("build").about("build the application for production"),
	);
	let matches = app.get_matches();
	match matches.subcommand() {
		("dev", Some(_)) => dev().await,
		("build", Some(_)) => build().await,
		_ => unreachable!(),
	}
}

async fn dev() -> Result<()> {
	// get host and port
	let host = std::env::var("HOST")
		.map(|host| host.parse().expect("HOST environment variable invalid"))
		.unwrap_or_else(|_| "0.0.0.0".parse().unwrap());
	let port = std::env::var("PORT")
		.map(|port| port.parse().expect("PORT environment variable invalid"))
		.unwrap_or_else(|_| 80);
	let addr = std::net::SocketAddr::new(host, port);

	let pinwheel = Pinwheel::dev(PathBuf::from("."), PathBuf::from("dist"));
	let pinwheel = Arc::new(pinwheel);

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
		let pinwheel = pinwheel.clone();
		let service = service_fn(move |request| {
			let pinwheel = pinwheel.clone();
			AssertUnwindSafe(async move {
				Result::<_, hyper::Error>::Ok(pinwheel.clone().handle(request).await)
			})
			.catch_unwind()
			.map(|result| match result {
				Ok(response) => response,
				Err(_) => Ok(Response::builder()
					.status(StatusCode::INTERNAL_SERVER_ERROR)
					.body(Body::from("internal server error"))
					.unwrap()),
			})
		});
		tokio::spawn(http.serve_connection(socket, service).map(|r| {
			if let Err(e) = r {
				eprintln!("http error: {}", e);
			}
		}));
	}
}

async fn build() -> Result<()> {
	pinwheel::build(Path::new("."), Path::new("dist"))?;
	Ok(())
}
