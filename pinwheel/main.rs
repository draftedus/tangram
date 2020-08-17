use anyhow::Result;
use futures::FutureExt;
use hyper::{service::service_fn, Body, Response, StatusCode};
use pinwheel::Pinwheel;
use std::{panic::AssertUnwindSafe, path::Path, sync::Arc};

pub fn main() {
	let mut app = clap::App::new("pinwheel")
		.version(clap::crate_version!())
		.setting(clap::AppSettings::SubcommandRequiredElseHelp);
	app = app.subcommand(
		clap::SubCommand::with_name("dev")
			.about("run the development server")
			.arg(
				clap::Arg::with_name("src-dir")
					.long("src-dir")
					.default_value(".")
					.takes_value(true),
			)
			.arg(
				clap::Arg::with_name("dst-dir")
					.long("dst-dir")
					.default_value("dist")
					.takes_value(true),
			),
	);
	app = app.subcommand(
		clap::SubCommand::with_name("build")
			.about("build the application for production")
			.arg(
				clap::Arg::with_name("src-dir")
					.long("src-dir")
					.default_value(".")
					.takes_value(true),
			)
			.arg(
				clap::Arg::with_name("dst-dir")
					.long("dst-dir")
					.default_value("dist")
					.takes_value(true),
			),
	);
	let mut runtime = tokio::runtime::Runtime::new().unwrap();
	let matches = app.get_matches();
	let result = match matches.subcommand() {
		("dev", Some(dev_matches)) => {
			let src_dir = dev_matches.value_of("src-dir").map(Path::new).unwrap();
			let dst_dir = dev_matches.value_of("dst-dir").map(Path::new).unwrap();
			runtime.block_on(dev(src_dir, dst_dir))
		}
		("build", Some(build_matches)) => {
			let src_dir = build_matches.value_of("src-dir").map(Path::new).unwrap();
			let dst_dir = build_matches.value_of("dst-dir").map(Path::new).unwrap();
			runtime.block_on(build(src_dir, dst_dir))
		}
		_ => unreachable!(),
	};
	result.unwrap();
}

async fn dev(src_dir: &Path, dst_dir: &Path) -> Result<()> {
	// get host and port
	let host = std::env::var("HOST")
		.map(|host| host.parse().expect("HOST environment variable invalid"))
		.unwrap_or_else(|_| "0.0.0.0".parse().unwrap());
	let port = std::env::var("PORT")
		.map(|port| port.parse().expect("PORT environment variable invalid"))
		.unwrap_or_else(|_| 80);
	let addr = std::net::SocketAddr::new(host, port);

	let pinwheel = Pinwheel::dev(src_dir.to_owned(), dst_dir.to_owned());
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
			async move {
				let result = AssertUnwindSafe(pinwheel.clone().handle(request))
					.catch_unwind()
					.await;
				match result {
					Ok(response) => response,
					Err(_) => {
						let response = Response::builder()
							.status(StatusCode::INTERNAL_SERVER_ERROR)
							.body(Body::from("internal server error"))
							.unwrap();
						Ok(response)
					}
				}
			}
		});
		tokio::spawn(http.serve_connection(socket, service).map(|r| {
			if let Err(e) = r {
				eprintln!("http error: {}", e);
			}
		}));
	}
}

async fn build(src_dir: &Path, dst_dir: &Path) -> Result<()> {
	pinwheel::build(src_dir, dst_dir)?;
	Ok(())
}
