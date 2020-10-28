fn main() {
	let cmd = if cfg!(not(windows)) { "npx" } else { "npx.exe" };
	let args = vec!["esbuild".to_owned(), "--help".to_owned()];
	let mut process = std::process::Command::new(dbg!(cmd))
		.stderr(std::process::Stdio::inherit())
		.args(&dbg!(args))
		.spawn()
		.unwrap();
	process.wait().unwrap();
}
