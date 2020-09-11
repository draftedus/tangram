use anyhow::Result;
use ndarray::prelude::*;
use std::path::Path;
use std::time::Instant;
use tangram::{dataframe::*, metrics};

fn main() -> Result<()> {
	// load the data
	let csv_file_path = Path::new("data/boston.csv");
	let nrows_train = 405;
	let _nrows_test = 101;

	let target_column_index = 13;
	let mut csv_reader = csv::Reader::from_path(csv_file_path)?;
	let options = FromCsvOptions {
		..Default::default()
	};
	let mut features = DataFrame::from_csv(&mut csv_reader, options, |_| {})?;
	let labels = features.columns.remove(target_column_index);
	let (dataframe_train, dataframe_test) = features.view().split_at_row(nrows_train);
	let (labels_train, labels_test) = labels.view().split_at_row(nrows_train);
	let labels_train = labels_train.as_number().unwrap();
	let labels_test = labels_test.as_number().unwrap();

	// compute stats
	let stats_settings = tangram::stats::StatsSettings {
		number_histogram_max_size: 100,
		text_histogram_max_size: 100,
	};
	// retrieve the column names
	let column_names: Vec<String> = dataframe_train
		.columns
		.iter()
		.map(|column| column.name().to_owned())
		.collect();

	let tangram::stats::ComputeStatsOutput {
		overall_column_stats,
		..
	} = tangram::stats::compute_stats(
		&column_names,
		&dataframe_train,
		&dataframe_test,
		&stats_settings,
		&mut |_| {},
	);
	let feature_groups = tangram::features::compute_feature_groups_tree(&overall_column_stats);
	let features_train =
		tangram::features::compute_features_dataframe(&dataframe_train, &feature_groups, &|| {});

	// train the model
	let train_options = tangram::tree::TrainOptions {
		learning_rate: 0.1,
		max_depth: 8,
		max_leaf_nodes: 255,
		max_rounds: 100,
		min_examples_leaf: 100,
		min_sum_hessians_in_leaf: 0.0,
		..Default::default()
	};
	let start = Instant::now();
	let model = tangram::tree::Regressor::train(
		features_train.view(),
		labels_train.clone(),
		train_options,
		&mut |_| {},
	);
	let end = Instant::now();
	println!("duration: {:?}", end - start);

	// compute rmse
	let n_features = features.ncols();

	let mut features_test = unsafe { Array2::uninitialized((dataframe_test.nrows(), n_features)) };
	tangram::features::compute_features_ndarray_value(
		&dataframe_test,
		&feature_groups,
		features_test.view_mut(),
		&|| {},
	);

	let mut predictions: Array1<f32> = unsafe { Array::uninitialized(features_test.nrows()) };
	model.predict(features_test.view(), predictions.view_mut(), None);
	let mse = metrics::mean_squared_error(predictions.view(), labels_test.data.into());
	println!("mse: {:?}", mse);

	Ok(())
}
