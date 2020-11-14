use tangram_util::{err, error::Result};

pub fn dev() -> Result<()> {
	watchexec::run::run(
		watchexec::ArgsBuilder::default()
			.paths(vec![std::path::PathBuf::from(".")])
			.cmd(vec![
				"cargo".to_owned(),
				"run".to_owned(),
				"--".to_owned(),
				"app".to_owned(),
			])
			.restart(true)
			.build()
			.map_err(|error| err!(error))?,
	)?;
	Ok(())
}
