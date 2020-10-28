use which::which;

fn main() {
	let cmd = which("npx").unwrap();
	let args = vec!["esbuild".to_owned(), "--help".to_owned()];
	let mut process = std::process::Command::new(dbg!(cmd))
		.stderr(std::process::Stdio::inherit())
		.args(&dbg!(args))
		.spawn()
		.unwrap();
	process.wait().unwrap();
}
