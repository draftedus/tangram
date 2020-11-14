use clap::Clap;
use pinwheel::Pinwheel;
use std::{path::PathBuf, sync::Arc};
use tangram_util::error::Result;

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
	match options {
		Options::Dev(options) => dev(options).unwrap(),
		Options::Build(options) => build(options).unwrap(),
	};
}

fn dev(options: DevOptions) -> Result<()> {
	let pinwheel = Pinwheel::dev(options.src_dir, options.dst_dir);
	tokio::runtime::Builder::new()
		.threaded_scheduler()
		.enable_all()
		.build()
		.unwrap()
		.block_on(pinwheel::serve(
			options.host,
			options.port,
			handle,
			pinwheel,
		))?;
	Ok(())
}

async fn handle(
	pinwheel: Arc<Pinwheel>,
	request: http::Request<hyper::Body>,
) -> http::Response<hyper::Body> {
	pinwheel.handle(request).await.unwrap()
}

fn build(options: BuildOptions) -> Result<()> {
	pinwheel::build(options.src_dir.as_path(), options.dst_dir.as_path())?;
	Ok(())
}
