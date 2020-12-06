use std::path::{Path, PathBuf};
// use tangram_util::serve::hash;
use rayon::prelude::*;
use tangram_util::{err, error::Result, serve::hash, zip};
use which::which;

fn main() -> Result<()> {
	let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
	let workspace_dir = crate_dir.parent().unwrap();
	let pages_dir = crate_dir.join("pages");
	let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
	let cargo_wasm_dir = workspace_dir.join("target_wasm");
	// Re-run this script if any non-ignored file in the workspace changes.
	for entry in ignore::Walk::new(&workspace_dir) {
		let entry = entry.unwrap();
		let path = entry.path();
		println!("cargo:rerun-if-changed={}", path.display());
	}
	std::fs::create_dir_all(out_dir.join("assets")).unwrap();
	std::fs::create_dir_all(out_dir.join("js")).unwrap();
	// Build client crates.
	let output_wasm_dir = out_dir.join("js");
	let mut client_crate_manifest_paths = Vec::new();
	for entry in ignore::Walk::new(&pages_dir) {
		let entry = entry.unwrap();
		let path = entry.path();
		if path.ends_with("client/Cargo.toml") {
			let client_crate_manifest_path = path.strip_prefix(workspace_dir).unwrap();
			client_crate_manifest_paths.push(client_crate_manifest_path.to_owned());
		}
	}
	let client_crate_package_names = client_crate_manifest_paths
		.iter()
		.map(|client_crate_manifest_path| {
			let client_crate_manifest =
				std::fs::read_to_string(&workspace_dir.join(client_crate_manifest_path))?;
			let client_crate_manifest: toml::Value = toml::from_str(&client_crate_manifest)?;
			let client_crate_name = client_crate_manifest
				.as_table()
				.unwrap()
				.get("package")
				.unwrap()
				.as_table()
				.unwrap()
				.get("name")
				.unwrap()
				.as_str()
				.unwrap()
				.to_owned();
			Ok(client_crate_name)
		})
		.collect::<Result<Vec<_>>>()?;
	let cmd = which("cargo")?;
	let mut args = vec![
		"build".to_owned(),
		"--target".to_owned(),
		"wasm32-unknown-unknown".to_owned(),
		"--target-dir".to_owned(),
		cargo_wasm_dir.to_str().unwrap().to_owned(),
	];
	if cfg!(not(debug_assertions)) {
		args.push("--release".to_owned())
	}
	for client_crate_package_name in client_crate_package_names.iter() {
		args.push("--package".to_owned());
		args.push(client_crate_package_name.clone());
	}
	let mut process = std::process::Command::new(cmd).args(&args).spawn()?;
	let status = process.wait()?;
	if !status.success() {
		return Err(err!("cargo {}", status.to_string()));
	}
	zip!(client_crate_manifest_paths, client_crate_package_names).for_each(
		|(client_crate_manifest_path, client_crate_package_name)| {
			let hash = hash(client_crate_manifest_path.to_str().unwrap());
			let input_wasm_path = format!(
				"{}/wasm32-unknown-unknown/{}/{}.wasm",
				cargo_wasm_dir.to_str().unwrap(),
				if cfg!(debug_assertions) {
					"debug"
				} else {
					"release"
				},
				client_crate_package_name,
			);
			let output_wasm_path = output_wasm_dir.join(format!("{}_bg.wasm", hash));
			// Do not re-run wasm-bindgen if the output wasm exists and is not older than the input wasm.
			let input_wasm_metadata = std::fs::metadata(&input_wasm_path).unwrap();
			let input_wasm_modified_time = input_wasm_metadata.modified().unwrap();
			if let Ok(output_wasm_metadata) = std::fs::metadata(&output_wasm_path) {
				let output_wasm_modified_time = output_wasm_metadata.modified().unwrap();
				if input_wasm_modified_time <= output_wasm_modified_time {
					return;
				}
			}
			wasm_bindgen_cli_support::Bindgen::new()
				.web(true)
				.unwrap()
				.keep_debug(cfg!(debug_assertions))
				.remove_producers_section(true)
				.remove_name_section(true)
				.input_path(input_wasm_path)
				.out_name(&hash)
				.generate(&output_wasm_dir)
				.map_err(|error| err!(error))
				.unwrap();
		},
	);

	// Collect the CSS.
	let mut css = String::new();
	for dir in ["app", "charts", "ui", "www"].iter() {
		let css_src_dir = workspace_dir.join(dir);
		for entry in ignore::Walk::new(&css_src_dir) {
			let entry = entry?;
			let path = entry.path();
			if path.extension().map(|e| e.to_str().unwrap()) == Some("css") {
				css.push_str(&std::fs::read_to_string(path)?);
			}
		}
	}
	std::fs::write(out_dir.join("styles.css"), css).unwrap();
	// // Copy static files in release mode.
	// if cfg!(not(debug_assertions)) {
	// 	let static_dir = crate_dir.join("static");
	// 	for entry in walkdir::WalkDir::new(&static_dir) {
	// 		let entry = entry.unwrap();
	// 		let path = entry.path();
	// 		if path.is_file() {
	// 			let out_path = dst_dir.join(path.strip_prefix(&static_dir).unwrap());
	// 			std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();
	// 			std::fs::copy(path, out_path).unwrap();
	// 		}
	// 	}
	// }
	// // Copy assets in release mode.
	// if cfg!(not(debug_assertions)) {
	// 	let asset_extensions = &["gif", "jpg", "png", "svg", "woff2"];
	// 	for entry in walkdir::WalkDir::new(&crate_dir) {
	// 		let entry = entry.unwrap();
	// 		let path = entry.path();
	// 		let extension = path.extension().map(|e| e.to_str().unwrap());
	// 		let extension = match extension {
	// 			Some(extension) => extension,
	// 			None => continue,
	// 		};
	// 		if !asset_extensions.contains(&extension) {
	// 			continue;
	// 		}
	// 		let asset_path = path.strip_prefix(workspace_dir).unwrap();
	// 		let hash = hash(asset_path.to_str().unwrap());
	// 		let asset_dst_path = dst_dir
	// 			.join("assets")
	// 			.join(&format!("{}.{}", hash, extension));
	// 		std::fs::copy(path, asset_dst_path).unwrap();
	// 	}
	// }
	Ok(())
}
