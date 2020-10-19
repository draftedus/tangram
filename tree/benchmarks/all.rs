#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
enum BenchmarkOutput {
	Regression(RegressionBenchmarkOutput),
	BinaryClassification(BinaryClassificationBenchmarkOutput),
	MulticlassClassification(MulticlassClassificationBenchmarkOutput),
}

impl std::fmt::Display for BenchmarkOutput {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		match self {
			BenchmarkOutput::Regression(r) => write!(f, "mse: {}", r.mse),
			BenchmarkOutput::BinaryClassification(r) => write!(f, "auc_roc: {}", r.auc_roc),
			BenchmarkOutput::MulticlassClassification(r) => write!(f, "accuracy: {}", r.accuracy),
		}
	}
}

#[derive(serde::Deserialize, Debug)]
struct RegressionBenchmarkOutput {
	mse: f32,
}

#[derive(serde::Deserialize, Debug)]
struct BinaryClassificationBenchmarkOutput {
	auc_roc: f32,
}

#[derive(serde::Deserialize, Debug)]
struct MulticlassClassificationBenchmarkOutput {
	accuracy: f32,
}

fn main() {
	// Test the regression datasets.
	println!("Regression");
	run_benchmarks(&["lightgbm", "xgboost", "sklearn", "tangram"], &["boston"]);
	println!();

	// Test the binary classification datasets.
	println!("Binary Classification");
	run_benchmarks(
		&["lightgbm", "xgboost", "sklearn", "tangram"],
		&["heart_disease", "census"],
	);
	run_benchmarks(&["lightgbm", "xgboost", "tangram"], &["higgs"]);
	run_benchmarks(&["lightgbm", "tangram"], &["flights"]);
	println!();

	// Test the multiclass classification datasets.
	println!("Multiclass Classification");
	run_benchmarks(&["lightgbm", "xgboost", "sklearn", "tangram"], &["iris"]);
	println!();
}

fn run_benchmarks(libraries: &[&str], datasets: &[&str]) {
	for dataset in datasets.iter() {
		println!("Testing {}", dataset);
		for library in libraries.iter() {
			let start = std::time::Instant::now();
			let output = if library == &"tangram" {
				run_tangram_tree_benchmark(dataset)
			} else {
				run_python_benchmark(dataset, library)
			};
			let duration = start.elapsed();
			println!("{} duration: {:?} {}", library, duration, output);
		}
	}
}

fn run_tangram_tree_benchmark(dataset: &str) -> BenchmarkOutput {
	let output = std::process::Command::new("cargo")
		.arg("run")
		.arg("--release")
		.arg("--bin")
		.arg(format!("tangram_tree_benchmark_{}", dataset))
		.output()
		.expect("failed to execute process");
	let output = serde_json::from_slice(output.stdout.as_slice()).unwrap();
	output
}

fn run_python_benchmark(dataset: &str, library: &str) -> BenchmarkOutput {
	let output = std::process::Command::new("python")
		.arg(format!("tree/benchmarks/{}.py", dataset))
		.arg("--library")
		.arg(library)
		.output()
		.expect("failed to execute process");
	let output = serde_json::from_slice(output.stdout.as_slice()).unwrap();
	output
}
