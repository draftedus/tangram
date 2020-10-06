use ndarray::prelude::*;
use std::path::Path;
use tangram_dataframe::*;
use tangram_metrics::StreamingMetric;

fn main() {
	// Load the data.
	let csv_file_path = Path::new("data/boston.csv");
	let n_rows_train = 405;
	let n_rows_test = 101;
	let target_column_index = 13;
	let mut dataframe = DataFrame::from_path(csv_file_path, Default::default(), |_| {}).unwrap();
	let labels = dataframe.columns.remove(target_column_index);
	let (dataframe_train, dataframe_test) = dataframe.view().split_at_row(n_rows_train);
	let (labels_train, labels_test) = labels.view().split_at_row(n_rows_train);
	let labels_train = labels_train.as_number().unwrap();
	let labels_test = labels_test.as_number().unwrap();

	let features_train = dataframe_train.to_rows_f32().unwrap();

	// Train the model.
	let model = tangram_linear::Regressor::train(
		features_train.view(),
		labels_train.clone(),
		&Default::default(),
		&mut |_| {},
	);

	// Make predictions on the test data.
	let features_test = dataframe_test.to_rows_f32().unwrap();
	let mut predictions: Array1<f32> = unsafe { Array::uninitialized(n_rows_test) };
	model.predict(features_test.view(), predictions.view_mut());

	// Compute Metrics.
	let mut metrics = tangram_metrics::RegressionMetrics::new();
	metrics.update(tangram_metrics::RegressionMetricsInput {
		predictions: predictions.as_slice().unwrap(),
		labels: labels_test.data,
	});
	let metrics = metrics.finalize();
	println!("{:?}", metrics);
}
