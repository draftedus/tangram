fn main() {
	let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
	println!("cargo:rustc-link-search={}", dir);
	if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
		println!("cargo:rustc-link-lib=static=tangram-linux-x64");
	} else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
		println!("cargo:rustc-link-lib=tangram-0.1.3-macos-x64");
	} else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
		println!("cargo:rustc-link-lib=tangram-0.1.3-windows-x64");
	} else {
		panic!("tangram-rust does not yet support your combination of operating system and CPU architecture. Want support for your platform? Get in touch at help@tangramhq.com.")
	}
}
