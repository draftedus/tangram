use ndarray::prelude::*;
use std::path::Path;
use tangram_dataframe::*;
use tangram_metrics::StreamingMetric;

fn main() {
	// load the data
	let csv_file_path = Path::new("data/boston.csv");
	let n_rows_train = 405;
	let n_rows_test = 101;

	// split into test and train
	let target_column_index = 13;
	let mut features = DataFrame::from_path(csv_file_path, Default::default(), |_| {}).unwrap();
	let labels = features.columns.remove(target_column_index);
	let (features_train, features_test) = features.view().split_at_row(n_rows_train);
	let (labels_train, labels_test) = labels.view().split_at_row(n_rows_train);
	let labels_train = labels_train.as_number().unwrap();
	let labels_test = labels_test.as_number().unwrap();

	// train the model
	let model = tangram_tree::Regressor::train(
		features_train,
		labels_train.clone(),
		Default::default(),
		&mut |_| {},
	);

	// make predictions on the test data
	let features = features_test.to_rows();
	let mut predictions: Array1<f32> = unsafe { Array::uninitialized(n_rows_test) };
	model.predict(features.view(), predictions.view_mut());

	// compute metrics
	let mut metrics = tangram_metrics::RegressionMetrics::new();
	metrics.update(tangram_metrics::RegressionMetricsInput {
		predictions: predictions.as_slice().unwrap(),
		labels: labels_test.data,
	});
	let metrics = metrics.finalize();
	println!("{:?}", metrics);
}
