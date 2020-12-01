use crate::watch::watch;
use clap::Clap;
use tangram_util::error::Result;

#[derive(Clap)]
#[clap(
	setting = clap::AppSettings::TrailingVarArg,
 )]
pub struct Args {
	#[clap(arg_enum)]
	target: Target,
	args: Vec<String>,
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
	let (cmd, cmd_args) = match args.target {
		Target::App => {
			let cmd = "cargo".to_owned();
			let mut cmd_args = vec![
				"run".to_owned(),
				"--bin".to_owned(),
				"tangram".to_owned(),
				"--".to_owned(),
				"app".to_owned(),
			];
			cmd_args.extend(args.args);
			(cmd, cmd_args)
		}
		Target::Www => {
			let cmd = "cargo".to_owned();
			let mut cmd_args = vec![
				"run".to_owned(),
				"--bin".to_owned(),
				"tangram_www".to_owned(),
				"--".to_owned(),
			];
			cmd_args.extend(args.args);
			(cmd, cmd_args)
		}
	};
	watch(watch_paths, ignore_paths, cmd, cmd_args)?;
	Ok(())
}
