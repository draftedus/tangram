use ndarray::prelude::*;
use std::path::Path;
use tangram_dataframe::*;
use tangram_metrics::StreamingMetric;

fn main() {
	// load the data
	let csv_file_path = Path::new("data/census.csv");
	let n_rows_train = 26049;
	let n_rows_test = 6512;
	let target_column_index = 14;
	let options = FromCsvOptions {
		..Default::default()
	};
	let mut features = DataFrame::from_path(csv_file_path, options, |_| {}).unwrap();
	let labels = features.columns.remove(target_column_index);
	let (features_train, features_test) = features.view().split_at_row(n_rows_train);
	let (labels_train, labels_test) = labels.view().split_at_row(n_rows_train);
	let labels_train = labels_train.as_enum().unwrap();
	let labels_test = labels_test.as_enum().unwrap();

	// train the model
	let train_options = tangram_tree::TrainOptions {
		learning_rate: 0.1,
		max_depth: 8,
		max_leaf_nodes: 255,
		max_rounds: 100,
		min_examples_per_leaf: 100,
		min_sum_hessians_in_leaf: 0.0,
		..Default::default()
	};
	let model = tangram_tree::BinaryClassifier::train(
		features_train,
		labels_train.clone(),
		train_options,
		&mut |_| {},
	);

	// make predictions on the test data
	let features_test = features_test.to_rows();
	let mut probabilities: Array2<f32> = unsafe { Array::uninitialized((n_rows_test, 2)) };
	model.predict(features_test.view(), probabilities.view_mut());

	// compute metrics
	let mut metrics = tangram_metrics::ClassificationMetrics::new(100);
	metrics.update(tangram_metrics::ClassificationMetricsInput {
		probabilities: probabilities.view(),
		labels: labels_test.data.into(),
	});
	let metrics = metrics.finalize();
	println!("{:?}", metrics.class_metrics[1]);
}
