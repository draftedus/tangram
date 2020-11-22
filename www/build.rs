use std::path::Path;
use tangram_deps::pinwheel;

fn main() {
	if cfg!(not(debug_assertions)) {
		let src_dir = Path::new(".");
		let dst_dir = Path::new("../build/pinwheel/www");
		// Remove the `dst_dir` if it exists and create it.
		if dst_dir.exists() {
			std::fs::remove_dir_all(&dst_dir).unwrap();
		}
		std::fs::create_dir_all(&dst_dir).unwrap();
		// Run the pinwheel build.
		pinwheel::build(&src_dir, &dst_dir).unwrap();
	}
}
