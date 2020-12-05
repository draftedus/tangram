use crate::watch::watch;
use clap::Clap;
use tangram_util::error::Result;
// use which::which;

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
	let workspace_dir = std::env::current_dir()?;
	let target_dir = workspace_dir.join("target");
	let target_wasm_dir = workspace_dir.join("target_wasm");
	let watch_paths = vec![workspace_dir];
	let ignore_paths = vec![target_dir, target_wasm_dir];
	let (cmd, cmd_args) = match args.target {
		Target::App => {
			let cmd = "cargo".to_owned();
			// let cmd = which("cargo")?.as_os_str().to_str().unwrap().to_owned();
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
