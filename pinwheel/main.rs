use clap::Clap;
use pinwheel::Pinwheel;
use std::{path::PathBuf, sync::Arc};
use tangram_util::error::Result;

#[derive(Clap)]
enum Args {
	Dev(DevArgs),
	Build(BuildArgs),
}

#[derive(Clap)]
struct DevArgs {
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
struct BuildArgs {
	#[clap(long, default_value = ".")]
	src_dir: PathBuf,
	#[clap(long, default_value = "dist")]
	dst_dir: PathBuf,
}

pub fn main() {
	let args = Args::parse();
	match args {
		Args::Dev(args) => dev(args).unwrap(),
		Args::Build(args) => build(args).unwrap(),
	};
}

fn dev(args: DevArgs) -> Result<()> {
	let pinwheel = Pinwheel::dev(args.src_dir, args.dst_dir);
	tokio::runtime::Builder::new()
		.threaded_scheduler()
		.enable_all()
		.build()
		.unwrap()
		.block_on(pinwheel::serve(args.host, args.port, handle, pinwheel))?;
	Ok(())
}

async fn handle(
	pinwheel: Arc<Pinwheel>,
	request: http::Request<hyper::Body>,
) -> http::Response<hyper::Body> {
	pinwheel.handle(request).await.unwrap()
}

fn build(args: BuildArgs) -> Result<()> {
	pinwheel::build(args.src_dir.as_path(), args.dst_dir.as_path())?;
	Ok(())
}
