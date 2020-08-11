use crate::{
	config::{self, Config},
	dataframe::*,
	features, gbt, grid,
	id::Id,
	linear, metrics,
	progress::{Progress, TrainProgress},
	stats, test, types,
	util::progress_counter::ProgressCounter,
};
use anyhow::{format_err, Context, Result};
use buffy::prelude::*;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use rayon::prelude::*;
use std::{collections::BTreeMap, path::Path};

pub fn train(
	model_id: Id,
	file_path: &Path,
	target_column_name: &str,
	config_path: Option<&Path>,
	update_progress: &mut dyn FnMut(Progress),
) -> Result<types::Model> {
	// load the config from the config file, if provided.
	let config: Option<Config> = load_config(config_path)?;

	// load the dataframe from the csv file
	let mut dataframe = load_dataframe(file_path, &config, update_progress)?;
	let row_count = dataframe.nrows();

	// retrieve the column names
	let column_names: Vec<String> = dataframe
		.columns
		.iter()
		.map(|column| column.name().to_owned())
		.collect();

	// shuffle the dataframe if enabled
	shuffle(&mut dataframe, &config, update_progress);

	// train test split
	let test_fraction = config
		.as_ref()
		.and_then(|config| config.test_fraction)
		.unwrap_or(0.2);
	let n_records_train = ((1.0 - test_fraction) * dataframe.nrows().to_f32().unwrap())
		.to_usize()
		.unwrap();
	let split_index = n_records_train;
	let (dataframe_train, dataframe_test) = dataframe.view().split_at_row(split_index);

	// compute stats
	let stats_settings = stats::StatsSettings {
		number_histogram_max_size: 100,
		text_histogram_max_size: 100,
	};
	let stats::ComputeStatsOutput {
		mut overall_column_stats,
		mut test_column_stats,
		mut train_column_stats,
	} = stats::compute_stats(
		&column_names,
		&dataframe_train,
		&dataframe_test,
		&stats_settings,
		&mut |stats_progress| update_progress(Progress::Stats(stats_progress)),
	);

	// find the target column
	let target_column_index = column_names
		.iter()
		.position(|column_name| *column_name == target_column_name)
		.ok_or_else(|| {
			format_err!(
				"did not find target column \"{}\" among column names \"{}\"",
				target_column_name,
				column_names.join(", ")
			)
		})?;

	// pull out the target column from the column stats
	let overall_target_column_stats = overall_column_stats.remove(target_column_index);
	let train_target_column_stats = train_column_stats.remove(target_column_index);
	let test_target_column_stats = test_column_stats.remove(target_column_index);

	// determine the task
	let task = match &overall_target_column_stats {
		stats::ColumnStats::Number(_) => Task::Regression,
		stats::ColumnStats::Enum(target_column) => Task::Classification {
			classes: target_column
				.histogram
				.iter()
				.map(|(option, _)| option.clone())
				.collect(),
		},
		_ => return Err(format_err!("invalid target column type")),
	};

	// split the train dataset into train and model comparison datasets
	let comparison_fraction = 0.1;
	let split_index = ((1.0 - comparison_fraction) * dataframe_train.nrows().to_f32().unwrap())
		.to_usize()
		.unwrap();
	let (dataframe_train, dataframe_comparison) = dataframe_train.split_at_row(split_index);

	// choose the comparison metric
	let comparison_metric = choose_comparison_metric(&config, &task)?;

	// create the hyperparameter grid
	let grid =
		compute_hyperparameter_grid(&config, &task, target_column_index, &overall_column_stats);

	// train each model in the grid and compute model comparison metrics
	let outputs: Vec<(TrainModelOutput, TestMetrics)> = grid
		.into_iter()
		.enumerate()
		.map(|(model_index, grid_item)| {
			update_progress(Progress::Training(model_index, None));
			let train_model_output = train_model(grid_item, &dataframe_train);
			let comparison_progress_counter =
				ProgressCounter::new(dataframe_comparison.nrows().to_u64().unwrap());
			update_progress(Progress::Training(
				model_index,
				Some(TrainProgress::ComputingModelComparisonMetrics(
					comparison_progress_counter,
				)),
			));
			let model_comparison_metrics = compute_model_comparison_metrics(
				&train_model_output,
				&dataframe_comparison,
				// &comparison_progress_counter,
			);
			(train_model_output, model_comparison_metrics)
		})
		.collect();

	// choose the best model
	let train_model_output = choose_best_model(outputs, &comparison_metric);

	// test the best model
	let progress = ProgressCounter::new(dataframe_test.nrows().to_u64().unwrap());
	update_progress(Progress::Testing(progress));
	let test_metrics = test_model(&train_model_output, &dataframe_test);

	// assemble the model
	let model = match task {
		Task::Regression => {
			let comparison_metric = match comparison_metric {
				ComparisonMetric::Regression(m) => m,
				_ => unreachable!(),
			};
			let test_metrics = match test_metrics {
				TestMetrics::Regression(m) => m,
				_ => unreachable!(),
			};
			let model = match train_model_output {
				TrainModelOutput::LinearRegressor(LinearRegressorTrainModelOutput {
					model,
					feature_groups,
					options,
					..
				}) => RegressionModel::Linear(LinearRegressor {
					feature_groups,
					options,
					model,
				}),
				TrainModelOutput::GBTRegressor(GBTRegressorTrainModelOutput {
					model,
					feature_groups,
					options,
					..
				}) => RegressionModel::Gbt(GBTRegressor {
					model,
					feature_groups,
					options,
				}),
				_ => unreachable!(),
			};
			types::Model::Regressor(types::Regressor {
				cached_size: Default::default(),
				unknown_fields: Default::default(),
				id: Field::Present(model_id.to_string()),
				target_column_name: Field::Present(target_column_name.to_string()),
				row_count: Field::Present(row_count.to_u64().unwrap()),
				stats_settings: Field::Present(stats_settings.into()),
				overall_column_stats: Field::Present(
					overall_column_stats.into_iter().map(Into::into).collect(),
				),
				overall_target_column_stats: Field::Present(overall_target_column_stats.into()),
				train_column_stats: Field::Present(
					train_column_stats.into_iter().map(Into::into).collect(),
				),
				train_target_column_stats: Field::Present(train_target_column_stats.into()),
				test_column_stats: Field::Present(
					test_column_stats.into_iter().map(Into::into).collect(),
				),
				test_target_column_stats: Field::Present(test_target_column_stats.into()),
				test_fraction: Field::Present(test_fraction),
				test_metrics: Field::Present(test_metrics.into()),
				model: Field::Present(model.into()),
				comparison_fraction: Field::Present(comparison_fraction),
				comparison_metric: Field::Present(comparison_metric.into()),
			})
		}
		Task::Classification { .. } => {
			let (test_metrics, model_test_metrics) = match test_metrics {
				TestMetrics::Classification(m) => m,
				_ => unreachable!(),
			};
			let comparison_metric = match comparison_metric {
				ComparisonMetric::Classification(m) => m,
				_ => unreachable!(),
			};
			let model = match train_model_output {
				TrainModelOutput::LinearBinaryClassifier(
					LinearBinaryClassifierTrainModelOutput {
						model,
						feature_groups,
						options,
						..
					},
				) => {
					let binary_classifier_model_test_metrics = model_test_metrics.unwrap();
					ClassificationModel::LinearBinary(LinearBinaryClassifier {
						auc_roc: binary_classifier_model_test_metrics.auc_roc,
						class_metrics: binary_classifier_model_test_metrics.class_metrics,
						feature_groups,
						model,
						options,
					})
				}
				TrainModelOutput::GBTBinaryClassifier(GBTBinaryClassifierTrainModelOutput {
					model,
					feature_groups,
					options,
					..
				}) => {
					let binary_classifier_model_test_metrics = model_test_metrics.unwrap();
					ClassificationModel::GbtBinary(GbtBinaryClassifier {
						auc_roc: binary_classifier_model_test_metrics.auc_roc,
						class_metrics: binary_classifier_model_test_metrics.class_metrics,
						feature_groups,
						model,
						options,
					})
				}
				TrainModelOutput::LinearMulticlassClassifier(
					LinearMulticlassClassifierTrainModelOutput {
						model,
						feature_groups,
						options,
						..
					},
				) => ClassificationModel::LinearMulticlass(LinearMulticlassClassifier {
					model,
					feature_groups,
					options,
				}),
				TrainModelOutput::GBTMulticlassClassifier(
					GBTMulticlassClassifierTrainModelOutput {
						model,
						feature_groups,
						options,
						..
					},
				) => ClassificationModel::GbtMulticlass(GbtMulticlassClassifier {
					model,
					feature_groups,
					options,
				}),
				_ => unreachable!(),
			};
			types::Model::Classifier(types::Classifier {
				cached_size: Default::default(),
				unknown_fields: Default::default(),
				id: Field::Present(model_id.to_string()),
				target_column_name: Field::Present(target_column_name.to_string()),
				row_count: Field::Present(row_count.to_u64().unwrap()),
				stats_settings: Field::Present(stats_settings.into()),
				overall_column_stats: Field::Present(
					overall_column_stats.into_iter().map(Into::into).collect(),
				),
				overall_target_column_stats: Field::Present(overall_target_column_stats.into()),
				train_column_stats: Field::Present(
					train_column_stats.into_iter().map(Into::into).collect(),
				),
				train_target_column_stats: Field::Present(train_target_column_stats.into()),
				test_column_stats: Field::Present(
					test_column_stats.into_iter().map(Into::into).collect(),
				),
				test_target_column_stats: Field::Present(test_target_column_stats.into()),
				test_fraction: Field::Present(test_fraction),
				test_metrics: Field::Present(test_metrics.into()),
				model: Field::Present(model.into()),
				comparison_fraction: Field::Present(comparison_fraction),
				comparison_metric: Field::Present(comparison_metric.into()),
			})
		}
	};

	Ok(model)
}

pub enum Task {
	Regression,
	Classification { classes: Vec<String> },
}

pub struct GbtBinaryClassifier {
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: gbt::BinaryClassifier,
	pub class_metrics: Vec<metrics::BinaryClassificationClassMetricsOutput>,
	pub auc_roc: f32,
	pub options: grid::GBTModelTrainOptions,
}

pub struct GbtMulticlassClassifier {
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: gbt::MulticlassClassifier,
	pub options: grid::GBTModelTrainOptions,
}

#[derive(Debug)]
pub enum ClassificationComparisonMetric {
	Accuracy,
	Aucroc,
	F1,
}

pub enum RegressionModel {
	Linear(LinearRegressor),
	Gbt(GBTRegressor),
}

pub struct LinearRegressor {
	pub feature_groups: Vec<features::FeatureGroup>,
	pub options: grid::LinearModelTrainOptions,
	pub model: linear::Regressor,
}

pub struct GBTRegressor {
	pub feature_groups: Vec<features::FeatureGroup>,
	pub options: grid::GBTModelTrainOptions,
	pub model: gbt::Regressor,
}

#[derive(Debug)]
pub enum RegressionComparisonMetric {
	MeanAbsoluteError,
	MeanSquaredError,
	RootMeanSquaredError,
	R2,
}

pub enum ClassificationModel {
	LinearBinary(LinearBinaryClassifier),
	LinearMulticlass(LinearMulticlassClassifier),
	GbtBinary(GbtBinaryClassifier),
	GbtMulticlass(GbtMulticlassClassifier),
}

pub struct LinearBinaryClassifier {
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: linear::BinaryClassifier,
	pub class_metrics: Vec<metrics::BinaryClassificationClassMetricsOutput>,
	pub auc_roc: f32,
	pub options: grid::LinearModelTrainOptions,
}

pub struct LinearMulticlassClassifier {
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: linear::MulticlassClassifier,
	pub options: grid::LinearModelTrainOptions,
}

#[derive(Debug)]
enum ComparisonMetric {
	Regression(RegressionComparisonMetric),
	Classification(ClassificationComparisonMetric),
}

#[derive(Debug)]
enum TestMetrics {
	Regression(metrics::RegressionMetricsOutput),
	Classification(
		(
			metrics::ClassificationMetricsOutput,
			Option<metrics::BinaryClassificationMetricsOutput>,
		),
	),
}

fn load_config(config_path: Option<&Path>) -> Result<Option<Config>> {
	if let Some(config_path) = config_path {
		let config = std::fs::read_to_string(config_path)
			.with_context(|| format!("failed to read config file {}", config_path.display()))?;
		let config = serde_yaml::from_str(&config)
			.with_context(|| format!("failed to parse config file {}", config_path.display()))?;
		Ok(Some(config))
	} else {
		Ok(None)
	}
}

fn load_dataframe(
	file_path: &Path,
	config: &Option<Config>,
	update_progress: &mut dyn FnMut(Progress),
) -> Result<DataFrame> {
	let len = std::fs::metadata(file_path)?.len();
	let progress_counter = ProgressCounter::new(len);
	update_progress(Progress::Loading(progress_counter.clone()));
	let mut csv_reader = csv::Reader::from_path(file_path)?;
	let column_types: Option<BTreeMap<String, ColumnType>> = config
		.as_ref()
		.and_then(|config| config.column_types.as_ref())
		.map(|column_types| {
			column_types
				.iter()
				.map(|(column_name, column_type)| {
					let column_type = match column_type {
						config::ColumnType::Unknown => ColumnType::Unknown,
						config::ColumnType::Number => ColumnType::Number,
						config::ColumnType::Enum { options } => ColumnType::Enum {
							options: options.clone(),
						},
						config::ColumnType::Text => ColumnType::Text,
					};
					(column_name.clone(), column_type)
				})
				.collect()
		});
	let dataframe = DataFrame::from_csv(
		&mut csv_reader,
		FromCsvOptions {
			column_types,
			infer_options: Default::default(),
		},
		|byte| progress_counter.set(byte),
	)?;
	Ok(dataframe)
}

fn shuffle(
	dataframe: &mut DataFrame,
	config: &Option<Config>,
	update_progress: &mut dyn FnMut(Progress),
) {
	// check if shuffling is enabled in the config
	// and use the seed from the config if provided
	let default_seed = 42;
	let shuffle_options = config
		.as_ref()
		.and_then(|config| config.shuffle.as_ref())
		.map(|shuffle| match shuffle {
			config::Shuffle::Enabled(enabled) => {
				if *enabled {
					Some(default_seed)
				} else {
					None
				}
			}
			config::Shuffle::Options { seed } => Some(*seed),
		})
		.unwrap_or(Some(default_seed));
	// shuffle the dataframe
	if let Some(seed) = shuffle_options {
		update_progress(Progress::Shuffling);
		dataframe.columns.par_iter_mut().for_each(|column| {
			let mut rng = Xoshiro256Plus::seed_from_u64(seed);
			match column {
				Column::Unknown(_) => {}
				Column::Number(column) => column.data.shuffle(&mut rng),
				Column::Enum(column) => column.data.shuffle(&mut rng),
				Column::Text(column) => column.data.shuffle(&mut rng),
			}
		});
	}
}

fn compute_hyperparameter_grid(
	config: &Option<Config>,
	task: &Task,
	target_column_index: usize,
	train_column_stats: &[stats::ColumnStats],
) -> Vec<grid::GridItem> {
	config
		.as_ref()
		.and_then(|config| config.grid.as_ref())
		.map(|grid| match &task {
			Task::Regression => grid::compute_regression_hyperparameter_grid(
				grid,
				target_column_index,
				&train_column_stats,
			),
			Task::Classification { classes } => match classes.len() {
				2 => grid::compute_binary_classification_hyperparameter_grid(
					grid,
					target_column_index,
					&train_column_stats,
				),
				_ => grid::compute_multiclass_classification_hyperparameter_grid(
					grid,
					target_column_index,
					&train_column_stats,
				),
			},
		})
		.unwrap_or_else(|| match &task {
			Task::Regression => grid::default_regression_hyperparameter_grid(
				target_column_index,
				&train_column_stats,
			),
			Task::Classification { classes } => match classes.len() {
				2 => grid::default_binary_classification_hyperparameter_grid(
					target_column_index,
					&train_column_stats,
				),
				_ => grid::default_multiclass_classification_hyperparameter_grid(
					target_column_index,
					&train_column_stats,
				),
			},
		})
}

enum TrainModelOutput {
	LinearRegressor(LinearRegressorTrainModelOutput),
	GBTRegressor(GBTRegressorTrainModelOutput),
	LinearBinaryClassifier(LinearBinaryClassifierTrainModelOutput),
	GBTBinaryClassifier(GBTBinaryClassifierTrainModelOutput),
	LinearMulticlassClassifier(LinearMulticlassClassifierTrainModelOutput),
	GBTMulticlassClassifier(GBTMulticlassClassifierTrainModelOutput),
}

struct LinearRegressorTrainModelOutput {
	model: linear::Regressor,
	feature_groups: Vec<features::FeatureGroup>,
	target_column_index: usize,
	options: grid::LinearModelTrainOptions,
}

struct GBTRegressorTrainModelOutput {
	model: gbt::Regressor,
	feature_groups: Vec<features::FeatureGroup>,
	target_column_index: usize,
	options: grid::GBTModelTrainOptions,
}

struct LinearBinaryClassifierTrainModelOutput {
	model: linear::BinaryClassifier,
	feature_groups: Vec<features::FeatureGroup>,
	target_column_index: usize,
	options: grid::LinearModelTrainOptions,
}

struct GBTBinaryClassifierTrainModelOutput {
	model: gbt::BinaryClassifier,
	feature_groups: Vec<features::FeatureGroup>,
	target_column_index: usize,
	options: grid::GBTModelTrainOptions,
}

struct LinearMulticlassClassifierTrainModelOutput {
	model: linear::MulticlassClassifier,
	feature_groups: Vec<features::FeatureGroup>,
	target_column_index: usize,
	options: grid::LinearModelTrainOptions,
}

struct GBTMulticlassClassifierTrainModelOutput {
	model: gbt::MulticlassClassifier,
	feature_groups: Vec<features::FeatureGroup>,
	target_column_index: usize,
	options: grid::GBTModelTrainOptions,
}

fn train_model(grid_item: grid::GridItem, dataframe_train: &DataFrameView) -> TrainModelOutput {
	match grid_item {
		grid::GridItem::LinearRegressor {
			target_column_index,
			feature_groups,
			options,
		} => train_linear_regressor(
			dataframe_train,
			target_column_index,
			feature_groups,
			options,
		),
		grid::GridItem::GBTRegressor {
			target_column_index,
			feature_groups,
			options,
		} => train_gbt_regressor(
			dataframe_train,
			target_column_index,
			feature_groups,
			options,
		),
		grid::GridItem::LinearBinaryClassifier {
			target_column_index,
			feature_groups,
			options,
		} => train_linear_binary_classifier(
			dataframe_train,
			target_column_index,
			feature_groups,
			options,
		),
		grid::GridItem::GBTBinaryClassifier {
			target_column_index,
			feature_groups,
			options,
		} => train_gbt_binary_classifier(
			dataframe_train,
			target_column_index,
			feature_groups,
			options,
		),
		grid::GridItem::LinearMulticlassClassifier {
			target_column_index,
			feature_groups,
			options,
		} => train_linear_multiclass_classifier(
			dataframe_train,
			target_column_index,
			feature_groups,
			options,
		),
		grid::GridItem::GBTMulticlassClassifier {
			target_column_index,
			feature_groups,
			options,
		} => train_gbt_multiclass_classifier(
			dataframe_train,
			target_column_index,
			feature_groups,
			options,
		),
	}
}

fn train_linear_regressor(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<features::FeatureGroup>,
	options: grid::LinearModelTrainOptions,
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = unsafe { Array2::uninitialized((dataframe_train.nrows(), n_features)) };
	features::compute_features_ndarray(dataframe_train, &feature_groups, features.view_mut());
	let labels = dataframe_train
		.columns
		.get(target_column_index)
		.unwrap()
		.as_number()
		.unwrap();
	let linear_options = linear::TrainOptions {
		early_stopping_fraction: options.early_stopping_fraction,
		l2_regularization: options.l2_regularization,
		learning_rate: options.learning_rate,
		max_epochs: options.max_epochs.to_usize().unwrap(),
		n_examples_per_batch: options.n_examples_per_batch.to_usize().unwrap(),
	};
	let model = linear::Regressor::train(features.view(), &labels, &linear_options);
	TrainModelOutput::LinearRegressor(LinearRegressorTrainModelOutput {
		model,
		feature_groups,
		target_column_index,
		options,
	})
}

fn train_gbt_regressor(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<features::FeatureGroup>,
	options: grid::GBTModelTrainOptions,
) -> TrainModelOutput {
	let features = features::compute_features_dataframe(dataframe_train, &feature_groups);
	let labels = dataframe_train
		.columns
		.get(target_column_index)
		.unwrap()
		.as_number()
		.unwrap()
		.clone();
	let base: usize = 2;
	let gbt_options = gbt::TrainOptions {
		compute_loss: true,
		early_stopping_options: None,
		l2_regularization: 0.0,
		learning_rate: options.learning_rate,
		max_depth: options.max_depth.to_usize().unwrap(),
		max_leaf_nodes: base.pow(options.max_depth.to_u32().unwrap()),
		max_non_missing_bins: 255,
		subsample_for_binning: 200_000,
		max_rounds: options.max_rounds.to_usize().unwrap(),
		min_examples_leaf: 20,
		min_sum_hessians_in_leaf: 1e-3,
		min_gain_to_split: 0.0,
		discrete_smoothing_factor: 10.0,
		discrete_l2_regularization: 10.0,
		discrete_min_examples_per_branch: 100,
	};
	let model = gbt::Regressor::train(features.view(), labels, gbt_options);
	TrainModelOutput::GBTRegressor(GBTRegressorTrainModelOutput {
		model,
		feature_groups,
		target_column_index,
		options,
	})
}

fn train_linear_binary_classifier(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<features::FeatureGroup>,
	options: grid::LinearModelTrainOptions,
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = unsafe { Array2::uninitialized((dataframe_train.nrows(), n_features)) };
	features::compute_features_ndarray(dataframe_train, &feature_groups, features.view_mut());
	let labels = dataframe_train
		.columns
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let linear_options = linear::TrainOptions {
		early_stopping_fraction: options.early_stopping_fraction,
		l2_regularization: options.l2_regularization,
		learning_rate: options.learning_rate,
		max_epochs: options.max_epochs.to_usize().unwrap(),
		n_examples_per_batch: options.n_examples_per_batch.to_usize().unwrap(),
	};
	let model = linear::BinaryClassifier::train(features.view(), &labels, &linear_options);
	TrainModelOutput::LinearBinaryClassifier(LinearBinaryClassifierTrainModelOutput {
		model,
		feature_groups,
		target_column_index,
		options,
	})
}

fn train_gbt_binary_classifier(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<features::FeatureGroup>,
	options: grid::GBTModelTrainOptions,
) -> TrainModelOutput {
	let features = features::compute_features_dataframe(dataframe_train, &feature_groups);
	let labels = dataframe_train
		.columns
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap()
		.clone();
	let base: usize = 2;
	let gbt_options = gbt::TrainOptions {
		compute_loss: true,
		discrete_l2_regularization: 10.0,
		discrete_min_examples_per_branch: 100,
		discrete_smoothing_factor: 10.0,
		early_stopping_options: None,
		l2_regularization: 0.0,
		learning_rate: options.learning_rate,
		max_depth: options.max_depth.to_usize().unwrap(),
		max_leaf_nodes: base.pow(options.max_depth.to_u32().unwrap()),
		max_non_missing_bins: 255,
		max_rounds: options.max_rounds.to_usize().unwrap(),
		min_examples_leaf: options.min_examples_per_leaf.to_usize().unwrap(),
		min_gain_to_split: 0.0,
		min_sum_hessians_in_leaf: 1e-3,
		subsample_for_binning: 200_000,
	};
	let model = gbt::BinaryClassifier::train(features.view(), labels, gbt_options);
	TrainModelOutput::GBTBinaryClassifier(GBTBinaryClassifierTrainModelOutput {
		model,
		feature_groups,
		target_column_index,
		options,
	})
}

fn train_linear_multiclass_classifier(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<features::FeatureGroup>,
	options: grid::LinearModelTrainOptions,
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = unsafe { Array2::uninitialized((dataframe_train.nrows(), n_features)) };
	features::compute_features_ndarray(dataframe_train, &feature_groups, features.view_mut());
	let labels = dataframe_train
		.columns
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let linear_options = linear::TrainOptions {
		early_stopping_fraction: options.early_stopping_fraction,
		l2_regularization: options.l2_regularization,
		learning_rate: options.learning_rate,
		max_epochs: options.max_epochs.to_usize().unwrap(),
		n_examples_per_batch: options.n_examples_per_batch.to_usize().unwrap(),
	};
	let model = linear::MulticlassClassifier::train(features.view(), &labels, &linear_options);
	TrainModelOutput::LinearMulticlassClassifier(LinearMulticlassClassifierTrainModelOutput {
		model,
		feature_groups,
		target_column_index,
		options,
	})
}

fn train_gbt_multiclass_classifier(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<features::FeatureGroup>,
	options: grid::GBTModelTrainOptions,
) -> TrainModelOutput {
	let features = features::compute_features_dataframe(dataframe_train, &feature_groups);
	let labels = dataframe_train
		.columns
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap()
		.clone();
	let base: usize = 2;
	let gbt_options = gbt::TrainOptions {
		compute_loss: true,
		early_stopping_options: None,
		l2_regularization: 0.0,
		learning_rate: options.learning_rate,
		max_depth: options.max_depth.to_usize().unwrap(),
		max_leaf_nodes: base.pow(options.max_depth.to_u32().unwrap()),
		max_non_missing_bins: 255,
		subsample_for_binning: 200_000,
		max_rounds: options.max_rounds.to_usize().unwrap(),
		min_examples_leaf: options.min_examples_per_leaf.to_usize().unwrap(),
		min_sum_hessians_in_leaf: 1e-3,
		min_gain_to_split: 0.0,
		discrete_smoothing_factor: 10.0,
		discrete_l2_regularization: 10.0,
		discrete_min_examples_per_branch: 100,
	};
	let model = gbt::MulticlassClassifier::train(features.view(), labels, gbt_options);
	TrainModelOutput::GBTMulticlassClassifier(GBTMulticlassClassifierTrainModelOutput {
		model,
		feature_groups,
		target_column_index,
		options,
	})
}

fn choose_comparison_metric(config: &Option<Config>, task: &Task) -> Result<ComparisonMetric> {
	match task {
		Task::Regression => {
			if let Some(metric) = config
				.as_ref()
				.and_then(|config| config.comparison_metric.as_ref())
			{
				match metric {
					config::ComparisonMetric::MAE => Ok(ComparisonMetric::Regression(
						RegressionComparisonMetric::MeanAbsoluteError,
					)),
					config::ComparisonMetric::MSE => Ok(ComparisonMetric::Regression(
						RegressionComparisonMetric::MeanSquaredError,
					)),
					config::ComparisonMetric::RMSE => Ok(ComparisonMetric::Regression(
						RegressionComparisonMetric::RootMeanSquaredError,
					)),
					config::ComparisonMetric::R2 => {
						Ok(ComparisonMetric::Regression(RegressionComparisonMetric::R2))
					}
					metric => Err(format_err!(
						"{} is an invalid model comparison metric for regression",
						metric
					)),
				}
			} else {
				Ok(ComparisonMetric::Regression(
					RegressionComparisonMetric::RootMeanSquaredError,
				))
			}
		}
		Task::Classification { classes } => {
			if let Some(metric) = config
				.as_ref()
				.and_then(|config| config.comparison_metric.as_ref())
			{
				let n_classes = classes.len();
				match (metric, n_classes) {
					(config::ComparisonMetric::Accuracy, 2) => Ok(
						ComparisonMetric::Classification(ClassificationComparisonMetric::Accuracy),
					),
					(config::ComparisonMetric::AUC, 2) => Ok(ComparisonMetric::Classification(
						ClassificationComparisonMetric::Aucroc,
					)),
					(config::ComparisonMetric::F1, 2) => Ok(ComparisonMetric::Classification(
						ClassificationComparisonMetric::F1,
					)),
					(config::ComparisonMetric::Accuracy, _) => Ok(
						ComparisonMetric::Classification(ClassificationComparisonMetric::Accuracy),
					),
					(metric, n_classes) => Err(format_err!(
						"{} is an invalid model comparison metric for classification with {} classes",
						metric, n_classes
					)),
				}
			} else {
				Ok(ComparisonMetric::Classification(
					ClassificationComparisonMetric::Accuracy,
				))
			}
		}
	}
}

fn compute_model_comparison_metrics(
	train_model_output: &TrainModelOutput,
	dataframe_comparison: &DataFrameView,
	// progress: &ProgressCounter,
) -> TestMetrics {
	match train_model_output {
		TrainModelOutput::LinearRegressor(train_model_output) => {
			let LinearRegressorTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			TestMetrics::Regression(test::test_linear_regressor(
				&dataframe_comparison,
				*target_column_index,
				feature_groups,
				model,
				// progress,
			))
		}
		TrainModelOutput::GBTRegressor(train_model_output) => {
			let GBTRegressorTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			TestMetrics::Regression(test::test_gbt_regressor(
				&dataframe_comparison,
				*target_column_index,
				feature_groups,
				model,
				// progress,
			))
		}
		TrainModelOutput::LinearBinaryClassifier(train_model_output) => {
			let LinearBinaryClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let (metrics, model_metrics) = test::test_linear_binary_classifier(
				&dataframe_comparison,
				*target_column_index,
				feature_groups,
				model,
				// progress,
			);
			TestMetrics::Classification((metrics, Some(model_metrics)))
		}
		TrainModelOutput::GBTBinaryClassifier(train_model_output) => {
			let GBTBinaryClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let (metrics, model_metrics) = test::test_gbt_binary_classifier(
				&dataframe_comparison,
				*target_column_index,
				feature_groups,
				model,
				// progress,
			);
			TestMetrics::Classification((metrics, Some(model_metrics)))
		}
		TrainModelOutput::LinearMulticlassClassifier(train_model_output) => {
			let LinearMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			TestMetrics::Classification((
				test::test_linear_multiclass_classifier(
					&dataframe_comparison,
					*target_column_index,
					feature_groups,
					model,
					// progress,
				),
				None,
			))
		}
		TrainModelOutput::GBTMulticlassClassifier(train_model_output) => {
			let GBTMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			TestMetrics::Classification((
				test::test_gbt_multiclass_classifier(
					&dataframe_comparison,
					*target_column_index,
					feature_groups,
					model,
					// progress,
				),
				None,
			))
		}
	}
}

fn choose_best_model(
	outputs: Vec<(TrainModelOutput, TestMetrics)>,
	comparison_metric: &ComparisonMetric,
) -> TrainModelOutput {
	match comparison_metric {
		ComparisonMetric::Regression(comparison_metric) => {
			choose_best_model_regression(outputs, comparison_metric)
		}
		ComparisonMetric::Classification(comparison_metric) => {
			choose_best_model_classification(outputs, comparison_metric)
		}
	}
}

fn choose_best_model_regression(
	outputs: Vec<(TrainModelOutput, TestMetrics)>,
	comparison_metric: &RegressionComparisonMetric,
) -> TrainModelOutput {
	outputs
		.into_iter()
		.max_by(|(_, metrics_a), (_, metrics_b)| {
			let metrics_a = match metrics_a {
				TestMetrics::Regression(m) => m,
				_ => unreachable!(),
			};
			let metrics_b = match metrics_b {
				TestMetrics::Regression(m) => m,
				_ => unreachable!(),
			};
			match comparison_metric {
				RegressionComparisonMetric::MeanAbsoluteError => {
					metrics_b.mae.partial_cmp(&metrics_a.mae).unwrap()
				}
				RegressionComparisonMetric::RootMeanSquaredError => {
					metrics_b.rmse.partial_cmp(&metrics_a.rmse).unwrap()
				}
				RegressionComparisonMetric::MeanSquaredError => {
					metrics_b.mse.partial_cmp(&metrics_a.mse).unwrap()
				}
				RegressionComparisonMetric::R2 => metrics_a.r2.partial_cmp(&metrics_b.r2).unwrap(),
			}
		})
		.map(|(model, _)| model)
		.unwrap()
}

fn choose_best_model_classification(
	outputs: Vec<(TrainModelOutput, TestMetrics)>,
	comparison_metric: &ClassificationComparisonMetric,
) -> TrainModelOutput {
	outputs
		.into_iter()
		.max_by(|(_, metrics_a), (_, metrics_b)| {
			let (task_metrics_a, model_metrics_a) = match metrics_a {
				TestMetrics::Classification(m) => m,
				_ => unreachable!(),
			};
			let (task_metrics_b, model_metrics_b) = match metrics_b {
				TestMetrics::Classification(m) => m,
				_ => unreachable!(),
			};
			match comparison_metric {
				ClassificationComparisonMetric::Accuracy => task_metrics_a
					.accuracy
					.partial_cmp(&task_metrics_b.accuracy)
					.unwrap(),
				ClassificationComparisonMetric::Aucroc => model_metrics_a
					.as_ref()
					.unwrap()
					.auc_roc
					.partial_cmp(&model_metrics_b.as_ref().unwrap().auc_roc)
					.unwrap(),
				ClassificationComparisonMetric::F1 => todo!(),
			}
		})
		.map(|(model, _)| model)
		.unwrap()
}

fn test_model(
	train_model_output: &TrainModelOutput,
	dataframe_test: &DataFrameView,
	// progress: &ProgressCounter,
) -> TestMetrics {
	match train_model_output {
		TrainModelOutput::LinearRegressor(train_model_output) => {
			let LinearRegressorTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			TestMetrics::Regression(test::test_linear_regressor(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				// progress,
			))
		}
		TrainModelOutput::GBTRegressor(train_model_output) => {
			let GBTRegressorTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			TestMetrics::Regression(test::test_gbt_regressor(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				// progress,
			))
		}
		TrainModelOutput::LinearBinaryClassifier(train_model_output) => {
			let LinearBinaryClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let (metrics, model_metrics) = test::test_linear_binary_classifier(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				// progress,
			);
			TestMetrics::Classification((metrics, Some(model_metrics)))
		}
		TrainModelOutput::GBTBinaryClassifier(train_model_output) => {
			let GBTBinaryClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let (metrics, model_metrics) = test::test_gbt_binary_classifier(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				// progress,
			);
			TestMetrics::Classification((metrics, Some(model_metrics)))
		}
		TrainModelOutput::LinearMulticlassClassifier(train_model_output) => {
			let LinearMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let metrics = test::test_linear_multiclass_classifier(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				// progress,
			);
			TestMetrics::Classification((metrics, None))
		}
		TrainModelOutput::GBTMulticlassClassifier(train_model_output) => {
			let GBTMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let metrics = test::test_gbt_multiclass_classifier(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				// progress,
			);
			TestMetrics::Classification((metrics, None))
		}
	}
}

impl Into<types::StatsSettings> for stats::StatsSettings {
	fn into(self) -> types::StatsSettings {
		types::StatsSettings {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			text_histogram_max_size: Field::Present(self.text_histogram_max_size.to_u64().unwrap()),
			number_histogram_max_size: Field::Present(
				self.number_histogram_max_size.to_u64().unwrap(),
			),
		}
	}
}

impl Into<types::FeatureGroup> for features::FeatureGroup {
	fn into(self) -> types::FeatureGroup {
		match self {
			Self::Identity(f) => types::FeatureGroup::Identity(f.into()),
			Self::Normalized(f) => types::FeatureGroup::Normalized(f.into()),
			Self::OneHotEncoded(f) => types::FeatureGroup::OneHotEncoded(f.into()),
			Self::BagOfWords(f) => types::FeatureGroup::BagOfWords(f.into()),
		}
	}
}

impl Into<types::IdentityFeatureGroup> for features::IdentityFeatureGroup {
	fn into(self) -> types::IdentityFeatureGroup {
		types::IdentityFeatureGroup {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			source_column_name: Present(self.source_column_name),
		}
	}
}

impl Into<types::NormalizedFeatureGroup> for features::NormalizedFeatureGroup {
	fn into(self) -> types::NormalizedFeatureGroup {
		types::NormalizedFeatureGroup {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			source_column_name: Present(self.source_column_name),
			mean: Present(self.mean),
			variance: Present(self.variance),
		}
	}
}

impl Into<types::OneHotEncodedFeatureGroup> for features::OneHotEncodedFeatureGroup {
	fn into(self) -> types::OneHotEncodedFeatureGroup {
		types::OneHotEncodedFeatureGroup {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			source_column_name: Present(self.source_column_name),
			categories: Present(self.categories),
		}
	}
}

impl Into<types::BagOfWordsFeatureGroup> for features::BagOfWordsFeatureGroup {
	fn into(self) -> types::BagOfWordsFeatureGroup {
		types::BagOfWordsFeatureGroup {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			source_column_name: Present(self.source_column_name),
			tokenizer: Present(self.tokenizer.into()),
			tokens: Present(self.tokens),
		}
	}
}

impl Into<types::Tokenizer> for features::Tokenizer {
	fn into(self) -> types::Tokenizer {
		match self {
			Self::Alphanumeric => types::Tokenizer::Alphanumeric,
		}
	}
}

impl Into<types::ColumnStats> for stats::ColumnStats {
	fn into(self) -> types::ColumnStats {
		match self {
			Self::Unknown(c) => types::ColumnStats::Unknown(c.into()),
			Self::Number(c) => types::ColumnStats::Number(c.into()),
			Self::Enum(c) => types::ColumnStats::Enum(c.into()),
			Self::Text(c) => types::ColumnStats::Text(c.into()),
		}
	}
}

impl Into<types::UnknownColumnStats> for stats::UnknownColumnStats {
	fn into(self) -> types::UnknownColumnStats {
		types::UnknownColumnStats {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			column_name: Present(self.column_name),
		}
	}
}

impl Into<types::NumberColumnStats> for stats::NumberColumnStats {
	fn into(self) -> types::NumberColumnStats {
		types::NumberColumnStats {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			column_name: Present(self.column_name),
			histogram: Present(self.histogram),
			invalid_count: Present(self.invalid_count),
			unique_count: Present(self.unique_count),
			min: Present(self.min),
			max: Present(self.max),
			mean: Present(self.mean),
			variance: Present(self.variance),
			std: Present(self.std),
			p25: Present(self.p25),
			p50: Present(self.p50),
			p75: Present(self.p75),
		}
	}
}

impl Into<types::EnumColumnStats> for stats::EnumColumnStats {
	fn into(self) -> types::EnumColumnStats {
		types::EnumColumnStats {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			column_name: Field::Present(self.column_name),
			histogram: Field::Present(
				self.histogram
					.into_iter()
					.map(|(s, v)| (s, v.to_u64().unwrap()))
					.collect(),
			),
			invalid_count: Field::Present(self.invalid_count.to_u64().unwrap()),
			unique_count: Field::Present(self.unique_count.to_u64().unwrap()),
		}
	}
}

impl Into<types::TextColumnStats> for stats::TextColumnStats {
	fn into(self) -> types::TextColumnStats {
		types::TextColumnStats {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			column_name: Present(self.column_name),
			top_tokens: Present(
				self.top_tokens
					.into_iter()
					.map(|(token, count, _)| (token, count))
					.collect(),
			),
		}
	}
}

impl Into<types::RegressionMetrics> for metrics::RegressionMetricsOutput {
	fn into(self) -> types::RegressionMetrics {
		types::RegressionMetrics {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			mse: Present(self.mse),
			rmse: Present(self.rmse),
			mae: Present(self.mae),
			r2: Present(self.r2),
			baseline_mse: Present(self.baseline_mse),
			baseline_rmse: Present(self.baseline_rmse),
		}
	}
}

impl Into<types::ClassificationMetrics> for metrics::ClassificationMetricsOutput {
	fn into(self) -> types::ClassificationMetrics {
		types::ClassificationMetrics {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			class_metrics: Present(self.class_metrics.into_iter().map(Into::into).collect()),
			accuracy: Present(self.accuracy),
			precision_unweighted: Present(self.precision_unweighted),
			precision_weighted: Present(self.precision_weighted),
			recall_unweighted: Present(self.recall_unweighted),
			recall_weighted: Present(self.recall_weighted),
			baseline_accuracy: Present(self.baseline_accuracy),
		}
	}
}

impl Into<types::ClassMetrics> for metrics::ClassMetrics {
	fn into(self) -> types::ClassMetrics {
		types::ClassMetrics {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			true_positives: Present(self.true_positives),
			false_positives: Present(self.false_positives),
			true_negatives: Present(self.true_negatives),
			false_negatives: Present(self.false_negatives),
			accuracy: Present(self.accuracy),
			precision: Present(self.precision),
			recall: Present(self.recall),
			f1_score: Present(self.f1_score),
		}
	}
}

impl Into<types::Tree> for crate::gbt::Tree {
	fn into(self) -> types::Tree {
		types::Tree {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			nodes: Present(self.nodes.into_iter().map(Into::into).collect()),
		}
	}
}

impl Into<types::Node> for crate::gbt::Node {
	fn into(self) -> types::Node {
		match self {
			Self::Branch(branch) => types::Node::Branch(branch.into()),
			Self::Leaf(leaf) => types::Node::Leaf(leaf.into()),
		}
	}
}

impl Into<types::BranchNode> for crate::gbt::BranchNode {
	fn into(self) -> types::BranchNode {
		types::BranchNode {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			left_child_index: Present(self.left_child_index.to_u64().unwrap()),
			right_child_index: Present(self.right_child_index.to_u64().unwrap()),
			split: Present(self.split.into()),
			examples_fraction: Present(self.examples_fraction),
		}
	}
}

impl Into<types::BranchSplit> for crate::gbt::BranchSplit {
	fn into(self) -> types::BranchSplit {
		match self {
			Self::Continuous(value) => types::BranchSplit::Continuous(value.into()),
			Self::Discrete(value) => types::BranchSplit::Discrete(value.into()),
		}
	}
}

impl Into<types::BranchSplitContinuous> for crate::gbt::BranchSplitContinuous {
	fn into(self) -> types::BranchSplitContinuous {
		let invalid_values_direction = match self.invalid_values_direction {
			crate::gbt::SplitDirection::Left => false,
			crate::gbt::SplitDirection::Right => true,
		};
		types::BranchSplitContinuous {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			feature_index: Present(self.feature_index.to_u64().unwrap()),
			split_value: Present(self.split_value),
			invalid_values_direction: Present(invalid_values_direction),
		}
	}
}

impl Into<types::BranchSplitDiscrete> for crate::gbt::BranchSplitDiscrete {
	fn into(self) -> types::BranchSplitDiscrete {
		let directions: Vec<bool> = (0..self.directions.n)
			.map(|i| self.directions.get(i).unwrap())
			.collect();
		types::BranchSplitDiscrete {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			feature_index: Present(self.feature_index.to_u64().unwrap()),
			directions: Present(directions),
		}
	}
}

impl Into<types::LeafNode> for crate::gbt::LeafNode {
	fn into(self) -> types::LeafNode {
		types::LeafNode {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			value: Present(self.value),
			examples_fraction: Present(self.examples_fraction),
		}
	}
}

impl Into<types::ClassificationModel> for ClassificationModel {
	fn into(self) -> types::ClassificationModel {
		match self {
			Self::LinearBinary(m) => types::ClassificationModel::LinearBinary(m.into()),
			Self::LinearMulticlass(m) => types::ClassificationModel::LinearMulticlass(m.into()),
			Self::GbtBinary(m) => types::ClassificationModel::GbtBinary(m.into()),
			Self::GbtMulticlass(m) => types::ClassificationModel::GbtMulticlass(m.into()),
		}
	}
}

impl Into<types::LinearBinaryClassifier> for LinearBinaryClassifier {
	fn into(self) -> types::LinearBinaryClassifier {
		types::LinearBinaryClassifier {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			feature_groups: Field::Present(
				self.feature_groups.into_iter().map(|f| f.into()).collect(),
			),
			options: Field::Present(self.options.into()),
			class_metrics: Field::Present(self.class_metrics.into_iter().map(Into::into).collect()),
			auc_roc: Field::Present(self.auc_roc),
			means: Field::Present(self.model.means.into_raw_vec()),
			weights: Field::Present(self.model.weights.into_raw_vec()),
			bias: Field::Present(self.model.bias),
			losses: Field::Present(self.model.losses.into_raw_vec()),
			classes: Field::Present(self.model.classes.into_raw_vec()),
		}
	}
}

impl Into<types::LinearModelTrainOptions> for grid::LinearModelTrainOptions {
	fn into(self) -> types::LinearModelTrainOptions {
		types::LinearModelTrainOptions {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			early_stopping_fraction: Present(self.early_stopping_fraction),
			l2_regularization: Present(self.l2_regularization),
			learning_rate: Present(self.learning_rate),
			max_epochs: Present(self.max_epochs),
			n_examples_per_batch: Present(self.n_examples_per_batch),
		}
	}
}

impl Into<types::GbtModelTrainOptions> for grid::GBTModelTrainOptions {
	fn into(self) -> types::GbtModelTrainOptions {
		types::GbtModelTrainOptions {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			depth: Present(self.max_depth),
			learning_rate: Present(self.learning_rate),
			min_examples_per_leaf: Present(self.min_examples_per_leaf),
			max_rounds: Present(self.max_rounds),
			early_stopping_fraction: Present(self.early_stopping_fraction),
		}
	}
}

impl Into<types::LinearMulticlassClassifier> for LinearMulticlassClassifier {
	fn into(self) -> types::LinearMulticlassClassifier {
		types::LinearMulticlassClassifier {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			feature_groups: Field::Present(
				self.feature_groups.into_iter().map(|f| f.into()).collect(),
			),
			n_features: Field::Present(self.model.weights.nrows().to_u64().unwrap()),
			n_classes: Field::Present(self.model.weights.ncols().to_u64().unwrap()),
			weights: Field::Present(self.model.weights.into_raw_vec()),
			biases: Field::Present(self.model.biases.into_raw_vec()),
			losses: Field::Present(self.model.losses.into_raw_vec()),
			options: Field::Present(self.options.into()),
			classes: Field::Present(self.model.classes.into_raw_vec()),
			means: Field::Present(self.model.means.into_raw_vec()),
		}
	}
}

impl Into<types::GbtBinaryClassifier> for GbtBinaryClassifier {
	fn into(self) -> types::GbtBinaryClassifier {
		let losses = self.model.losses.map(|l| l.into_raw_vec()).unwrap();
		let trees = self.model.trees.into_iter().map(Into::into).collect();
		let class_metrics = self.class_metrics.into_iter().map(Into::into).collect();
		let feature_importances = self.model.feature_importances.unwrap().into_raw_vec();
		let options = self.options.into();
		types::GbtBinaryClassifier {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			feature_groups: Field::Present(
				self.feature_groups.into_iter().map(|f| f.into()).collect(),
			),
			trees: Field::Present(trees),
			class_metrics: Field::Present(class_metrics),
			bias: Field::Present(self.model.bias),
			losses: Field::Present(losses),
			auc_roc: Field::Present(self.auc_roc),
			classes: Field::Present(self.model.classes),
			feature_importances: Field::Present(feature_importances),
			options: Field::Present(options),
		}
	}
}

impl Into<types::GbtMulticlassClassifier> for GbtMulticlassClassifier {
	fn into(self) -> types::GbtMulticlassClassifier {
		types::GbtMulticlassClassifier {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			n_rounds: Field::Present(self.model.n_rounds.to_u64().unwrap()),
			n_classes: Field::Present(self.model.n_classes.to_u64().unwrap()),
			biases: Field::Present(self.model.biases),
			options: Field::Present(self.options.into()),
			trees: Field::Present(self.model.trees.into_iter().map(|t| t.into()).collect()),
			feature_groups: Field::Present(
				self.feature_groups.into_iter().map(|t| t.into()).collect(),
			),
			losses: Field::Present(self.model.losses.unwrap().into_raw_vec()),
			classes: Field::Present(self.model.classes),
			feature_importances: Field::Present(
				self.model.feature_importances.unwrap().into_raw_vec(),
			),
		}
	}
}

impl Into<types::BinaryClassifierClassMetrics> for metrics::BinaryClassificationClassMetricsOutput {
	fn into(self) -> types::BinaryClassifierClassMetrics {
		types::BinaryClassifierClassMetrics {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			thresholds: Present(self.thresholds.into_iter().map(Into::into).collect()),
		}
	}
}

impl Into<types::ThresholdMetrics> for metrics::BinaryClassificationThresholdMetricsOutput {
	fn into(self) -> types::ThresholdMetrics {
		types::ThresholdMetrics {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			threshold: Present(self.threshold),
			true_positives: Present(self.true_positives),
			false_positives: Present(self.false_positives),
			true_negatives: Present(self.true_negatives),
			false_negatives: Present(self.false_negatives),
			accuracy: Present(self.accuracy),
			precision: Present(self.precision),
			recall: Present(self.recall),
			f1_score: Present(self.f1_score),
			true_positive_rate: Present(self.true_positive_rate),
			false_positive_rate: Present(self.false_positive_rate),
		}
	}
}

impl Into<types::ClassificationComparisonMetric> for ClassificationComparisonMetric {
	fn into(self) -> types::ClassificationComparisonMetric {
		match self {
			Self::Accuracy => types::ClassificationComparisonMetric::Accuracy,
			Self::Aucroc => types::ClassificationComparisonMetric::Aucroc,
			Self::F1 => types::ClassificationComparisonMetric::F1,
		}
	}
}

impl Into<types::RegressionModel> for RegressionModel {
	fn into(self) -> types::RegressionModel {
		match self {
			Self::Linear(m) => types::RegressionModel::Linear(m.into()),
			Self::Gbt(m) => types::RegressionModel::Gbt(m.into()),
		}
	}
}

impl Into<types::LinearRegressor> for LinearRegressor {
	fn into(self) -> types::LinearRegressor {
		types::LinearRegressor {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			feature_groups: Field::Present(
				self.feature_groups.into_iter().map(|f| f.into()).collect(),
			),
			weights: Field::Present(self.model.weights.into_raw_vec()),
			bias: Field::Present(self.model.bias),
			losses: Field::Present(self.model.losses.into_raw_vec()),
			means: Field::Present(self.model.means.into_raw_vec()),
			options: Field::Present(self.options.into()),
		}
	}
}

impl Into<types::GbtRegressor> for GBTRegressor {
	fn into(self) -> types::GbtRegressor {
		let losses = self.model.losses.map(|l| l.into_raw_vec()).unwrap();
		let trees = self.model.trees.into_iter().map(Into::into).collect();
		types::GbtRegressor {
			cached_size: Default::default(),
			unknown_fields: Default::default(),
			feature_groups: Field::Present(
				self.feature_groups.into_iter().map(|f| f.into()).collect(),
			),
			trees: Field::Present(trees),
			bias: Field::Present(self.model.bias),
			losses: Field::Present(losses),
			options: Field::Present(self.options.into()),
			feature_importances: Field::Present(
				self.model.feature_importances.unwrap().into_raw_vec(),
			),
		}
	}
}

impl Into<types::RegressionComparisonMetric> for RegressionComparisonMetric {
	fn into(self) -> types::RegressionComparisonMetric {
		match self {
			Self::MeanAbsoluteError => types::RegressionComparisonMetric::MeanAbsoluteError,
			Self::MeanSquaredError => types::RegressionComparisonMetric::MeanSquaredError,
			Self::RootMeanSquaredError => types::RegressionComparisonMetric::RootMeanSquaredError,
			Self::R2 => types::RegressionComparisonMetric::R2,
		}
	}
}
