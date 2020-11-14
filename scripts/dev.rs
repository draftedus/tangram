fn main() {
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
			.unwrap(),
	)
	.unwrap()
}
