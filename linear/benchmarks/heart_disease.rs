use ndarray::prelude::*;
use serde_json::json;
use std::path::Path;
use tangram_dataframe::prelude::*;
use tangram_metrics::Metric;
use tangram_util::zip;

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/heart_disease_train.csv");
	let csv_file_path_test = Path::new("data/heart_disease_test.csv");
	let target_column_index = 13;
	let mut features_train =
		DataFrame::from_path(csv_file_path_train, Default::default(), |_| {}).unwrap();
	let labels_train = features_train.columns_mut().remove(target_column_index);
	let labels_train = labels_train.as_enum().unwrap();
	let mut features_test =
		DataFrame::from_path(csv_file_path_test, Default::default(), |_| {}).unwrap();
	let labels_test = features_test.columns_mut().remove(target_column_index);
	let labels_test = labels_test.as_enum().unwrap();
	let feature_groups: Vec<tangram_features::FeatureGroup> = features_train
		.columns()
		.iter()
		.map(|column| {
			tangram_features::FeatureGroup::Normalized(
				tangram_features::NormalizedFeatureGroup::fit(column.view()),
			)
		})
		.collect();
	let features_train = tangram_features::compute_features_array_f32(
		&features_train.view(),
		feature_groups.as_slice(),
		&|| {},
	);
	let features_test = tangram_features::compute_features_array_f32(
		&features_test.view(),
		feature_groups.as_slice(),
		&|| {},
	);

	// Train the model.
	let train_output = tangram_linear::BinaryClassifier::train(
		features_train.view(),
		labels_train.view(),
		&tangram_linear::TrainOptions {
			learning_rate: 0.01,
			max_epochs: 1,
			n_examples_per_batch: 1,
			..Default::default()
		},
		&mut |_| {},
	);

	// Make predictions on the test data.
	let mut probabilities = Array::zeros(labels_test.len());
	train_output
		.model
		.predict(features_test.view(), probabilities.view_mut());

	// Compute metrics.
	let input = zip!(probabilities.iter(), labels_test.iter())
		.map(|(probability, label)| (*probability, label.unwrap()))
		.collect();
	let auc_roc = tangram_metrics::AUCROC::compute(input);

	// Compute memory usage.
	let mut memory = None;
	let file = std::fs::read_to_string("/proc/self/status").unwrap();
	for line in file.lines() {
		if line.starts_with("VmHWM") {
			memory = Some(line.split(':').nth(1).map(|x| x.trim().to_owned()).unwrap());
		}
	}

	let output = json!({
		"auc_roc": auc_roc,
		"memory": memory,
	});
	println!("{}", output);
}
