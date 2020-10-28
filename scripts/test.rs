fn main() {
	let cmd = if cfg!(not(target_os = "windows")) {
		"npx"
	} else {
		"npx.exe"
	};
	let args = vec!["esbuild".to_owned(), "--help".to_owned()];
	let mut process = std::process::Command::new(cmd)
		.stderr(std::process::Stdio::inherit())
		.args(&args)
		.spawn()
		.unwrap();
	let status = process.wait().unwrap();
	println!("{}", status);
}
