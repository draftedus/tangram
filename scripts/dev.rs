use crate::watch::watch;
use tangram_util::error::Result;

pub fn dev() -> Result<()> {
	let tangram_path = std::env::current_dir()?;
	let build_path = tangram_path.join("build");
	let watch_paths = vec![tangram_path];
	let ignore_paths = vec![build_path];
	let cmd = "cargo".to_owned();
	let args = vec!["run".to_owned(), "--".to_owned(), "app".to_owned()];
	watch(watch_paths, ignore_paths, cmd, args)?;
	Ok(())
}
