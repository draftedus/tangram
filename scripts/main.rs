use clap::Clap;
use tangram_util::error::Result;

mod dev;
mod generate_license;
mod prepare_release;
mod production_track_test;
mod watch;

#[derive(Clap)]
enum Args {
	Dev(self::dev::Args),
	GenerateLicense(self::generate_license::Args),
	PrepareRelease,
	ProductionTrackTest(self::production_track_test::Args),
}

#[tokio::main]
async fn main() -> Result<()> {
	let args = Args::parse();
	match args {
		Args::Dev(args) => self::dev::dev(args)?,
		Args::GenerateLicense(args) => self::generate_license::generate_license(args)?,
		Args::PrepareRelease => self::prepare_release::prepare_release().await?,
		Args::ProductionTrackTest(args) => {
			self::production_track_test::production_track_test(args).await?
		}
	}
	Ok(())
}
