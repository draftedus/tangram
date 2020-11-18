use tangram_deps::pinwheel;

fn main() {
	if cfg!(not(debug_assertions)) {
		let src_dir = std::env::current_dir().unwrap();
		let dst_dir = src_dir.join("../build/pinwheel/www");
		pinwheel::build(&src_dir, &dst_dir).unwrap();
	}
}
