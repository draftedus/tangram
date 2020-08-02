use std::path::PathBuf;

fn main() {
	if cfg!(not(debug_assertions)) {
		pinwheel::build(&PathBuf::from(""), &PathBuf::from("../target/js")).unwrap();
	}
}
