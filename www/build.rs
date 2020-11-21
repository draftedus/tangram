use std::path::Path;
use tangram_deps::pinwheel;

fn main() {
	if cfg!(not(debug_assertions)) {
		let src_dir = Path::new(".");
		let dst_dir = Path::new("../build/pinwheel/www");
		pinwheel::build(&src_dir, &dst_dir).unwrap();
	}
}
