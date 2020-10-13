use ndarray::prelude::*;
use std::path::Path;
use tangram_dataframe::prelude::*;
use tangram_metrics::Metric;
use tangram_metrics::StreamingMetric;

fn main() {
	// Load the data.
	let csv_file_path = Path::new("data/heart-disease.csv");
	let n_rows_train = 242;
	let n_rows_test = 61;
	let target_column_index = 13;
	let mut features = DataFrame::from_path(csv_file_path, Default::default(), |_| {}).unwrap();

	let labels = features.columns.remove(target_column_index);
	let (features_train, features_test) = features.view().split_at_row(n_rows_train);
	let (labels_train, labels_test) = labels.view().split_at_row(n_rows_train);
	let labels_train = labels_train.as_enum().unwrap();
	let labels_test = labels_test.as_enum().unwrap();

	// Train the model.
	let model = tangram_tree::BinaryClassifier::train(
		features_train,
		labels_train.clone(),
		tangram_tree::TrainOptions {
			max_leaf_nodes: 255,
			..Default::default()
		},
		&mut |_| {},
	);

	// Make predictions on the test data.
	let features_test = features_test.to_rows();
	let mut probabilities: Array2<f32> = unsafe { Array2::uninitialized((n_rows_test, 2)) };
	model.predict(features_test.view(), probabilities.view_mut());

	// Compute metrics.
	let input = probabilities
		.column(1)
		.iter()
		.cloned()
		.zip(labels_test.data.iter().map(|d| d.unwrap()))
		.collect();
	let auc_roc = tangram_metrics::AUCROC::compute(input);
	let mut metrics = tangram_metrics::BinaryClassificationMetrics::new(3);
	metrics.update(tangram_metrics::BinaryClassificationMetricsInput {
		probabilities: probabilities.view(),
		labels: labels_test.data.into(),
	});
	let metrics = metrics.finalize();
	println!("accuracy: {}", metrics.thresholds[1].accuracy);
	println!("auc: {}", auc_roc);
}
