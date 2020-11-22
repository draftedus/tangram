use crate::watch::watch;
use clap::Clap;
use tangram_util::error::Result;

#[derive(Clap)]
pub struct Args {
	#[clap(arg_enum)]
	target: Target,
}

#[derive(Clap)]
enum Target {
	#[clap(name = "app")]
	App,
	#[clap(name = "www")]
	Www,
}

pub fn dev(args: Args) -> Result<()> {
	let tangram_path = std::env::current_dir()?;
	let build_path = tangram_path.join("build");
	let watch_paths = vec![tangram_path];
	let ignore_paths = vec![build_path];
	let (cmd, args) = match args.target {
		Target::App => {
			let cmd = "cargo".to_owned();
			let args = vec!["run".to_owned(), "--".to_owned(), "app".to_owned()];
			(cmd, args)
		}
		Target::Www => {
			let cmd = "cargo".to_owned();
			let args = vec![
				"run".to_owned(),
				"--bin".to_owned(),
				"tangram_www".to_owned(),
			];
			(cmd, args)
		}
	};
	watch(watch_paths, ignore_paths, cmd, args)?;
	Ok(())
}
