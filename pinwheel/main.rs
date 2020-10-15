use anyhow::Result;
use clap::Clap;
use futures::FutureExt;
use hyper::{
	service::{make_service_fn, service_fn},
	Body, Response, StatusCode,
};
use pinwheel::Pinwheel;
use std::{convert::Infallible, panic::AssertUnwindSafe, path::PathBuf, sync::Arc};

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
	let mut runtime = tokio::runtime::Builder::new()
		.threaded_scheduler()
		.enable_all()
		.build()
		.unwrap();
	match options {
		Options::Dev(options) => runtime.block_on(dev(options)).unwrap(),
		Options::Build(options) => runtime.block_on(build(options)).unwrap(),
	};
}

async fn dev(options: DevOptions) -> Result<()> {
	let pinwheel = Arc::new(Pinwheel::dev(options.src_dir, options.dst_dir));
	let service = make_service_fn(|_| {
		let pinwheel = pinwheel.clone();
		async move {
			Ok::<_, Infallible>(service_fn(move |request| {
				let pinwheel = pinwheel.clone();
				async move {
					Ok::<_, Infallible>(
						AssertUnwindSafe(pinwheel.handle(request))
							.map(|result| result.unwrap())
							.catch_unwind()
							.await
							.unwrap_or_else(|_| {
								Response::builder()
									.status(StatusCode::INTERNAL_SERVER_ERROR)
									.body(Body::from("internal server error"))
									.unwrap()
							}),
					)
				}
			}))
		}
	});
	let addr = std::net::SocketAddr::new(options.host, options.port);
	let listener = std::net::TcpListener::bind(&addr)?;
	eprintln!("🚀 serving on port {}", options.port);
	hyper::Server::from_tcp(listener)?.serve(service).await?;
	Ok(())
}

async fn build(options: BuildOptions) -> Result<()> {
	pinwheel::build(options.src_dir.as_path(), options.dst_dir.as_path())?;
	Ok(())
}
