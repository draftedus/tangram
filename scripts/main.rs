use clap::Clap;
use tangram_util::error::Result;

mod dev;
mod generate_license;
mod prepare_release;
mod production_track_test;
mod watch;
mod www;

#[derive(Clap)]
enum Args {
	Dev,
	GenerateLicense(self::generate_license::Args),
	PrepareRelease,
	ProductionTrackTest(self::production_track_test::Args),
	WwwDev,
}

#[tokio::main]
async fn main() -> Result<()> {
	let args = Args::parse();
	match args {
		Args::Dev => self::dev::dev()?,
		Args::GenerateLicense(args) => self::generate_license::generate_license(args)?,
		Args::PrepareRelease => self::prepare_release::prepare_release().await?,
		Args::ProductionTrackTest(args) => {
			self::production_track_test::production_track_test(args).await?
		}
		Args::WwwDev => self::www::dev::dev()?,
	}
	Ok(())
}
