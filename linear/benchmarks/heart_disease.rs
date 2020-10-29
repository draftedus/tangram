use itertools::izip;
use ndarray::prelude::*;
use serde_json::json;
use std::path::Path;
use tangram_dataframe::prelude::*;
use tangram_metrics::{Metric, StreamingMetric};

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
		.map(|column| match column {
			DataFrameColumn::Number(column) => {
				let mean_variance =
					tangram_metrics::MeanVariance::compute(column.view().as_slice());
				tangram_features::FeatureGroup::Normalized(
					tangram_features::NormalizedFeatureGroup {
						source_column_name: column.name().clone().unwrap(),
						mean: mean_variance.mean,
						variance: mean_variance.variance,
					},
				)
			}
			DataFrameColumn::Enum(column) => tangram_features::FeatureGroup::Normalized(
				tangram_features::compute_normalized_feature_group_for_enum_column(column.view()),
			),
			_ => unreachable!(),
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
	let mut metrics = tangram_metrics::BinaryClassificationMetrics::new(3);
	metrics.update(tangram_metrics::BinaryClassificationMetricsInput {
		probabilities: probabilities.view().as_slice().unwrap(),
		labels: labels_test.view().data(),
	});
	let metrics = metrics.finalize();
	let input = izip!(probabilities.iter(), labels_test.iter())
		.map(|(probability, label)| (*probability, label.unwrap()))
		.collect();
	let auc_roc = tangram_metrics::AUCROC::compute(input);
	let output = json!({ "auc_roc": auc_roc, "accuracy": metrics.thresholds[metrics.thresholds.len() / 2].accuracy });
	println!("{}", output);
}
