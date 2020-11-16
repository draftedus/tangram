use ndarray::prelude::*;
use serde_json::json;
use std::path::Path;
use tangram_dataframe::prelude::*;
use tangram_metrics::StreamingMetric;

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/boston_train.csv");
	let csv_file_path_test = Path::new("data/boston_test.csv");
	let target_column_index = 13;
	let mut features_train =
		DataFrame::from_path(csv_file_path_train, Default::default(), |_| {}).unwrap();
	let labels_train = features_train.columns_mut().remove(target_column_index);
	let mut features_test =
		DataFrame::from_path(csv_file_path_test, Default::default(), |_| {}).unwrap();
	let labels_test = features_test.columns_mut().remove(target_column_index);
	let labels_train = labels_train.as_number().unwrap();
	let labels_test = labels_test.as_number().unwrap();

	// Train the model.
	let train_output = tangram_tree::Regressor::train(
		features_train.view(),
		labels_train.view(),
		&tangram_tree::TrainOptions {
			learning_rate: 0.1,
			max_leaf_nodes: 255,
			max_rounds: 100,
			..Default::default()
		},
		&mut |_| {},
	);

	// Make predictions on the test data.
	let features_test = features_test.to_rows();
	let mut predictions = Array::zeros(labels_test.len());
	train_output
		.model
		.predict(features_test.view(), predictions.view_mut());

	// Compute metrics.
	let mut metrics = tangram_metrics::RegressionMetrics::new();
	metrics.update(tangram_metrics::RegressionMetricsInput {
		predictions: predictions.as_slice().unwrap(),
		labels: labels_test.view().as_slice(),
	});
	let metrics = metrics.finalize();

	// Compute memory usage.
	let mut memory = None;
	let file = std::fs::read_to_string("/proc/self/status").unwrap();
	for line in file.lines() {
		if line.starts_with("VmHWM") {
			memory = Some(line.split(':').nth(1).map(|x| x.trim().to_owned()).unwrap());
		}
	}

	let output = json!({
		"mse": metrics.mse,
		"memory": memory,
	});
	println!("{}", output);
}
