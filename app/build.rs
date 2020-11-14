use std::path::Path;

fn main() {
	let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
	let workspace_dir = crate_dir.parent().unwrap();
	let pages_dir = crate_dir.join("pages");
	let dst_dir = workspace_dir.join("build/pinwheel/app");
	let cargo_wasm_dir = workspace_dir.join("build/cargo-wasm");
	// Remove the `dst_dir` if it exists and create it.
	if dst_dir.exists() {
		std::fs::remove_dir_all(&dst_dir).unwrap();
	}
	std::fs::create_dir_all(&dst_dir).unwrap();
	std::fs::create_dir(dst_dir.join("assets")).unwrap();
	std::fs::create_dir(dst_dir.join("js")).unwrap();
	// Build all the client crates.
	for entry in walkdir::WalkDir::new(&pages_dir) {
		let entry = entry.unwrap();
		let path = entry.path();
		if !path.ends_with("client/Cargo.toml") {
			continue;
		}
		for entry in walkdir::WalkDir::new(path.parent().unwrap()) {
			let entry = entry.unwrap();
			let path = entry.path();
			println!("cargo:rerun-if-changed={}", path.display());
		}
		let client_crate_manifest_path = path.strip_prefix(workspace_dir).unwrap();
		pinwheel::build_client_crate(
			workspace_dir,
			client_crate_manifest_path,
			&cargo_wasm_dir,
			false,
			&dst_dir,
		)
		.unwrap();
	}
	// Collect all the CSS.
	let mut css = String::new();
	let mut collect_css = |css_src_dir: &Path| {
		for entry in walkdir::WalkDir::new(&css_src_dir) {
			let entry = entry.unwrap();
			let path = entry.path();
			if entry.file_type().is_dir() {
				println!("cargo:rerun-if-changed={}", path.display());
			}
			if path.extension().map(|e| e.to_str().unwrap()) == Some("css") {
				println!("cargo:rerun-if-changed={}", path.display());
				css.push_str(&std::fs::read_to_string(path).unwrap());
			}
		}
	};
	collect_css(&workspace_dir.join("app"));
	collect_css(&workspace_dir.join("charts"));
	collect_css(&workspace_dir.join("ui"));
	collect_css(&workspace_dir.join("www"));
	std::fs::write(dst_dir.join("styles.css"), css).unwrap();
	// Copy all the static files in release.
	if cfg!(not(debug_assertions)) {
		let static_dir = crate_dir.join("static");
		for entry in walkdir::WalkDir::new(&static_dir) {
			let entry = entry.unwrap();
			let path = entry.path();
			if path.is_file() {
				let out_path = dst_dir.join(path.strip_prefix(&static_dir).unwrap());
				std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();
				std::fs::copy(path, out_path).unwrap();
			}
		}
	}
	// Copy all the assets in release.
	if cfg!(not(debug_assertions)) {
		let asset_extensions = &["gif", "jpg", "png", "svg", "woff2"];
		for entry in walkdir::WalkDir::new(&crate_dir) {
			let entry = entry.unwrap();
			let path = entry.path();
			let extension = path.extension().map(|e| e.to_str().unwrap());
			let extension = match extension {
				Some(extension) => extension,
				None => continue,
			};
			if !asset_extensions.contains(&extension) {
				continue;
			}
			println!("cargo:rerun-if-changed={}", path.display());
			let asset_path = path.strip_prefix(workspace_dir).unwrap();
			let hash = pinwheel::hash(asset_path.to_str().unwrap());
			let asset_dst_path = dst_dir
				.join("assets")
				.join(&format!("{}.{}", hash, extension));
			std::fs::copy(path, asset_dst_path).unwrap();
		}
	}
	// Run the pinwheel build in release
	if cfg!(not(debug_assertions)) {
		pinwheel::build(Path::new("."), Path::new("../build/pinwheel/app")).unwrap();
	}
}
