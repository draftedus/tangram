use itertools::izip;
use ndarray::prelude::*;
use serde_json::json;
use std::path::Path;
use tangram_dataframe::prelude::*;
use tangram_metrics::{Metric, StreamingMetric};

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/boston_train.csv");
	let csv_file_path_test = Path::new("data/boston_test.csv");
	let target_column_index = 13;
	let mut features_train =
		DataFrame::from_path(csv_file_path_train, Default::default(), |_| {}).unwrap();

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
			DataFrameColumn::Enum(column) => {
				tangram_features::FeatureGroup::Identity(tangram_features::IdentityFeatureGroup {
					source_column_name: column.name().clone().unwrap(),
				})
			}
			_ => unreachable!(),
		})
		.collect();

	let labels_train = features_train.columns_mut().remove(target_column_index);
	let labels_train = labels_train.as_number().unwrap();
	for (column, feature_group) in izip!(
		features_train.columns_mut().iter_mut(),
		feature_groups.iter()
	) {
		match feature_group {
			tangram_features::FeatureGroup::Normalized(feature_group) => match column {
				tangram_dataframe::DataFrameColumn::Number(column) => {
					feature_group.compute_dataframe(column.data_mut());
				}
				_ => unreachable!(),
			},
			tangram_features::FeatureGroup::Identity(_) => {
				// nothing to do
			}
			_ => unreachable!(),
		}
	}
	let features_train = features_train.to_rows_f32().unwrap();

	let mut features_test =
		DataFrame::from_path(csv_file_path_test, Default::default(), |_| {}).unwrap();
	let labels_test = features_test.columns_mut().remove(target_column_index);
	let labels_test = labels_test.as_number().unwrap();
	for (column, feature_group) in izip!(
		features_test.columns_mut().iter_mut(),
		feature_groups.iter()
	) {
		match feature_group {
			tangram_features::FeatureGroup::Normalized(feature_group) => match column {
				tangram_dataframe::DataFrameColumn::Number(column) => {
					feature_group.compute_dataframe(column.data_mut());
				}
				_ => unreachable!(),
			},
			tangram_features::FeatureGroup::Identity(_) => {
				// nothing to do
			}
			_ => unreachable!(),
		}
	}
	let features_test = features_test.to_rows_f32().unwrap();

	// Train the model.
	let train_output = tangram_linear::Regressor::train(
		features_train.view(),
		labels_train.view(),
		&tangram_linear::TrainOptions {
			learning_rate: 0.01,
			max_epochs: 1000,
			..Default::default()
		},
		&mut |_| {},
	);

	// Make predictions on the test data.
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
	let output = json!({"mse": metrics.mse});
	println!("{}", output);
}
