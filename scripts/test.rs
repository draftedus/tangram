fn main() {
	let cmd = "npx";
	let args = vec!["esbuild".to_owned()];
	let mut process = std::process::Command::new(cmd)
		.stderr(std::process::Stdio::inherit())
		.args(&args)
		.spawn()
		.unwrap();
	let status = process.wait().unwrap();
	println!("{}", status);
}
