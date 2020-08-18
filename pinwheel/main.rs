use anyhow::Result;
use clap::Clap;
use futures::FutureExt;
use hyper::{service::service_fn, Body, Response, StatusCode};
use pinwheel::Pinwheel;
use std::{panic::AssertUnwindSafe, path::PathBuf, sync::Arc};

#[derive(Clap)]
enum Options {
	Dev(DevOptions),
	Build(BuildOptions),
}

#[derive(Clap)]
struct DevOptions {
	#[clap(long, default_value = ".")]
	src_dir: PathBuf,
	#[clap(long, default_value = "dist")]
	dst_dir: PathBuf,
	#[clap(long, default_value = "0.0.0.0")]
	host: std::net::IpAddr,
	#[clap(long, default_value = "8080")]
	port: u16,
}

#[derive(Clap)]
struct BuildOptions {
	#[clap(long, default_value = ".")]
	src_dir: PathBuf,
	#[clap(long, default_value = "dist")]
	dst_dir: PathBuf,
}

pub fn main() {
	let options = Options::parse();
	let mut runtime = tokio::runtime::Runtime::new().unwrap();
	let result = match options {
		Options::Dev(options) => runtime.block_on(dev(options)),
		Options::Build(options) => runtime.block_on(build(options)),
	};
	result.unwrap();
}

async fn dev(options: DevOptions) -> Result<()> {
	let addr = std::net::SocketAddr::new(options.host, options.port);

	let pinwheel = Pinwheel::dev(options.src_dir, options.dst_dir);
	let pinwheel = Arc::new(pinwheel);

	// start the server
	let listener = std::net::TcpListener::bind(&addr).unwrap();
	let mut listener = tokio::net::TcpListener::from_std(listener).unwrap();
	let http = hyper::server::conn::Http::new();
	eprintln!("🚀 serving on port {}", options.port);

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

async fn build(options: BuildOptions) -> Result<()> {
	pinwheel::build(options.src_dir.as_path(), options.dst_dir.as_path())?;
	Ok(())
}
