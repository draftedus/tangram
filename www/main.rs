use tangram_deps::{pinwheel::Pinwheel, tokio};

#[tokio::main]
async fn main() {
	// Create the pinwheel.
	#[cfg(debug_assertions)]
	let pinwheel = Pinwheel::dev(
		std::path::PathBuf::from("www"),
		std::path::PathBuf::from("build/pinwheel/www"),
	);
	#[cfg(not(debug_assertions))]
	let pinwheel = Pinwheel::prod(tangram_deps::include_dir::include_dir!(
		"../build/pinwheel/www"
	));
	let host = "0.0.0.0".parse().unwrap();
	let port = 8080;
	pinwheel.serve(host, port).await.unwrap();
}
