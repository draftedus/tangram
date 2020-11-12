use std::path::{Path, PathBuf};

fn main() {
	let src_dir = Path::new(".");
	let dst_dir = Path::new("../build/pinwheel/app");
	let collect_css = |css_src_dir: &Path, output_file_name: &str| {
		let mut css = String::new();
		for path in walkdir::WalkDir::new(&css_src_dir) {
			let path = path.unwrap();
			let path = path.path();
			if path.extension().map(|e| e.to_str().unwrap()) == Some("css") {
				css.push_str(&std::fs::read_to_string(path).unwrap());
			}
		}
		std::fs::write(dst_dir.join(output_file_name), css).unwrap();
	};
	collect_css(&src_dir.join("../app"), "app.css");
	collect_css(&src_dir.join("../charts"), "charts.css");
	collect_css(&src_dir.join("../ui"), "ui.css");
	collect_css(&src_dir.join("../www"), "www.css");
	if cfg!(not(debug_assertions)) {
		pinwheel::build(
			&PathBuf::from("."),
			&PathBuf::from("../build/pinwheel/wasm/app"),
			&PathBuf::from("../build/pinwheel/app"),
		)
		.unwrap();
	}
}
