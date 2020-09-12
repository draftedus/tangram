use crate::{
	config::{self, Config},
	dataframe::*,
	features, grid, linear, metrics, model, stats, test, tree,
	util::{id::Id, progress_counter::ProgressCounter},
};
use anyhow::{format_err, Context, Result};
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use std::{collections::BTreeMap, path::Path};

pub fn train(
	model_id: Id,
	file_path: &Path,
	target_column_name: &str,
	config_path: Option<&Path>,
	update_progress: &mut dyn FnMut(Progress),
) -> Result<model::Model> {
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
	let num_models = grid.len();
	let outputs: Vec<(TrainModelOutput, TestMetrics)> = grid
		.into_iter()
		.enumerate()
		.map(|(model_index, grid_item)| {
			let train_model_output = train_model(grid_item, &dataframe_train, &mut |progress| {
				update_progress(Progress::Training(GridTrainProgress {
					current: model_index.to_u64().unwrap() + 1,
					total: num_models.to_u64().unwrap(),
					grid_item_progress: progress,
				}))
			});

			let model_comparison_metrics = compute_model_comparison_metrics(
				&train_model_output,
				&dataframe_comparison,
				&mut |progress| {
					update_progress(Progress::Training(GridTrainProgress {
						current: model_index.to_u64().unwrap() + 1,
						total: num_models.to_u64().unwrap(),
						grid_item_progress: TrainProgress::ComputingModelComparisonMetrics(
							progress,
						),
					}))
				},
			);
			(train_model_output, model_comparison_metrics)
		})
		.collect();

	// choose the best model
	let train_model_output = choose_best_model(outputs, &comparison_metric);

	// test the best model
	update_progress(Progress::Testing);
	let test_metrics = test_model(&train_model_output, &dataframe_test, &mut |_| {});

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
				TrainModelOutput::TreeRegressor(TreeRegressorTrainModelOutput {
					model,
					feature_groups,
					options,
					..
				}) => RegressionModel::Tree(TreeRegressor {
					model,
					feature_groups,
					options,
				}),
				_ => unreachable!(),
			};
			model::Model::Regressor(model::Regressor {
				id: model_id.to_string(),
				target_column_name: target_column_name.to_string(),
				row_count: row_count.to_u64().unwrap(),
				stats_settings: stats_settings.into(),
				overall_column_stats: overall_column_stats.into_iter().map(Into::into).collect(),
				overall_target_column_stats: overall_target_column_stats.into(),
				train_column_stats: train_column_stats.into_iter().map(Into::into).collect(),
				train_target_column_stats: train_target_column_stats.into(),
				test_column_stats: test_column_stats.into_iter().map(Into::into).collect(),
				test_target_column_stats: test_target_column_stats.into(),
				test_fraction,
				test_metrics: test_metrics.into(),
				model: model.into(),
				comparison_fraction,
				comparison_metric: comparison_metric.into(),
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
				TrainModelOutput::TreeBinaryClassifier(TreeBinaryClassifierTrainModelOutput {
					model,
					feature_groups,
					options,
					..
				}) => {
					let binary_classifier_model_test_metrics = model_test_metrics.unwrap();
					ClassificationModel::TreeBinary(TreeBinaryClassifier {
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
				TrainModelOutput::TreeMulticlassClassifier(
					TreeMulticlassClassifierTrainModelOutput {
						model,
						feature_groups,
						options,
						..
					},
				) => ClassificationModel::TreeMulticlass(TreeMulticlassClassifier {
					model,
					feature_groups,
					options,
				}),
				_ => unreachable!(),
			};
			model::Model::Classifier(model::Classifier {
				id: model_id.to_string(),
				target_column_name: target_column_name.to_string(),
				row_count: row_count.to_u64().unwrap(),
				stats_settings: stats_settings.into(),
				overall_column_stats: overall_column_stats.into_iter().map(Into::into).collect(),
				overall_target_column_stats: overall_target_column_stats.into(),
				train_column_stats: train_column_stats.into_iter().map(Into::into).collect(),
				train_target_column_stats: train_target_column_stats.into(),
				test_column_stats: test_column_stats.into_iter().map(Into::into).collect(),
				test_target_column_stats: test_target_column_stats.into(),
				test_fraction,
				test_metrics: test_metrics.into(),
				model: model.into(),
				comparison_fraction,
				comparison_metric: comparison_metric.into(),
			})
		}
	};

	Ok(model)
}

enum Task {
	Regression,
	Classification { classes: Vec<String> },
}

struct TreeBinaryClassifier {
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: tree::BinaryClassifier,
	pub class_metrics: Vec<metrics::BinaryClassificationClassMetricsOutput>,
	pub auc_roc: f32,
	pub options: grid::TreeModelTrainOptions,
}

struct TreeMulticlassClassifier {
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: tree::MulticlassClassifier,
	pub options: grid::TreeModelTrainOptions,
}

enum ClassificationComparisonMetric {
	Accuracy,
	Aucroc,
	F1,
}

enum RegressionModel {
	Linear(LinearRegressor),
	Tree(TreeRegressor),
}

struct LinearRegressor {
	pub feature_groups: Vec<features::FeatureGroup>,
	pub options: grid::LinearModelTrainOptions,
	pub model: linear::Regressor,
}

struct TreeRegressor {
	pub feature_groups: Vec<features::FeatureGroup>,
	pub options: grid::TreeModelTrainOptions,
	pub model: tree::Regressor,
}

enum RegressionComparisonMetric {
	MeanAbsoluteError,
	MeanSquaredError,
	RootMeanSquaredError,
	R2,
}

enum ClassificationModel {
	LinearBinary(LinearBinaryClassifier),
	LinearMulticlass(LinearMulticlassClassifier),
	TreeBinary(TreeBinaryClassifier),
	TreeMulticlass(TreeMulticlassClassifier),
}

struct LinearBinaryClassifier {
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: linear::BinaryClassifier,
	pub class_metrics: Vec<metrics::BinaryClassificationClassMetricsOutput>,
	pub auc_roc: f32,
	pub options: grid::LinearModelTrainOptions,
}

struct LinearMulticlassClassifier {
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: linear::MulticlassClassifier,
	pub options: grid::LinearModelTrainOptions,
}

enum ComparisonMetric {
	Regression(RegressionComparisonMetric),
	Classification(ClassificationComparisonMetric),
}

enum TestMetrics {
	Regression(metrics::RegressionMetricsOutput),
	Classification(
		(
			metrics::ClassificationMetricsOutput,
			Option<metrics::BinaryClassificationMetricsOutput>,
		),
	),
}

#[derive(Debug)]
pub enum Progress {
	Loading(ProgressCounter),
	Shuffling,
	Stats(StatsProgress),
	Training(GridTrainProgress),
	Testing,
}

#[derive(Debug)]
pub enum StatsProgress {
	DatasetStats(ProgressCounter),
	HistogramStats(ProgressCounter),
}

#[derive(Debug)]
pub struct GridTrainProgress {
	pub current: u64,
	pub total: u64,
	pub grid_item_progress: TrainProgress,
}

#[derive(Debug)]
pub enum TrainProgress {
	ComputingFeatures(ProgressCounter),
	TrainingModel(ModelTrainProgress),
	ComputingModelComparisonMetrics(ModelTestProgress),
}

#[derive(Clone, Debug)]
pub enum ModelTrainProgress {
	Linear(crate::linear::Progress),
	Tree(crate::tree::Progress),
}

#[derive(Clone, Debug)]
pub enum ModelTestProgress {
	ComputingFeatures(ProgressCounter),
	Testing,
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
		dataframe.columns.iter_mut().for_each(|column| {
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
	TreeRegressor(TreeRegressorTrainModelOutput),
	LinearBinaryClassifier(LinearBinaryClassifierTrainModelOutput),
	TreeBinaryClassifier(TreeBinaryClassifierTrainModelOutput),
	LinearMulticlassClassifier(LinearMulticlassClassifierTrainModelOutput),
	TreeMulticlassClassifier(TreeMulticlassClassifierTrainModelOutput),
}

struct LinearRegressorTrainModelOutput {
	model: linear::Regressor,
	feature_groups: Vec<features::FeatureGroup>,
	target_column_index: usize,
	options: grid::LinearModelTrainOptions,
}

struct TreeRegressorTrainModelOutput {
	model: tree::Regressor,
	feature_groups: Vec<features::FeatureGroup>,
	target_column_index: usize,
	options: grid::TreeModelTrainOptions,
}

struct LinearBinaryClassifierTrainModelOutput {
	model: linear::BinaryClassifier,
	feature_groups: Vec<features::FeatureGroup>,
	target_column_index: usize,
	options: grid::LinearModelTrainOptions,
}

struct TreeBinaryClassifierTrainModelOutput {
	model: tree::BinaryClassifier,
	feature_groups: Vec<features::FeatureGroup>,
	target_column_index: usize,
	options: grid::TreeModelTrainOptions,
}

struct LinearMulticlassClassifierTrainModelOutput {
	model: linear::MulticlassClassifier,
	feature_groups: Vec<features::FeatureGroup>,
	target_column_index: usize,
	options: grid::LinearModelTrainOptions,
}

struct TreeMulticlassClassifierTrainModelOutput {
	model: tree::MulticlassClassifier,
	feature_groups: Vec<features::FeatureGroup>,
	target_column_index: usize,
	options: grid::TreeModelTrainOptions,
}

fn train_model(
	grid_item: grid::GridItem,
	dataframe_train: &DataFrameView,
	update_progress: &mut dyn FnMut(TrainProgress),
) -> TrainModelOutput {
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
			update_progress,
		),
		grid::GridItem::TreeRegressor {
			target_column_index,
			feature_groups,
			options,
		} => train_tree_regressor(
			dataframe_train,
			target_column_index,
			feature_groups,
			options,
			update_progress,
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
			update_progress,
		),
		grid::GridItem::TreeBinaryClassifier {
			target_column_index,
			feature_groups,
			options,
		} => train_tree_binary_classifier(
			dataframe_train,
			target_column_index,
			feature_groups,
			options,
			update_progress,
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
			update_progress,
		),
		grid::GridItem::TreeMulticlassClassifier {
			target_column_index,
			feature_groups,
			options,
		} => train_tree_multiclass_classifier(
			dataframe_train,
			target_column_index,
			feature_groups,
			options,
			update_progress,
		),
	}
}

fn train_linear_regressor(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<features::FeatureGroup>,
	options: grid::LinearModelTrainOptions,
	update_progress: &mut dyn FnMut(TrainProgress),
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_counter = ProgressCounter::new(dataframe_train.nrows().to_u64().unwrap());
	update_progress(TrainProgress::ComputingFeatures(progress_counter.clone()));
	let mut features = unsafe { Array2::uninitialized((dataframe_train.nrows(), n_features)) };
	features::compute_features_ndarray(
		dataframe_train,
		&feature_groups,
		features.view_mut(),
		&|| progress_counter.inc(1),
	);
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
	let model =
		linear::Regressor::train(features.view(), &labels, &linear_options, &mut |progress| {
			update_progress(TrainProgress::TrainingModel(ModelTrainProgress::Linear(
				progress,
			)))
		});
	TrainModelOutput::LinearRegressor(LinearRegressorTrainModelOutput {
		model,
		feature_groups,
		target_column_index,
		options,
	})
}

fn train_tree_regressor(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<features::FeatureGroup>,
	options: grid::TreeModelTrainOptions,
	update_progress: &mut dyn FnMut(TrainProgress),
) -> TrainModelOutput {
	let progress_counter = ProgressCounter::new(dataframe_train.nrows().to_u64().unwrap());
	update_progress(TrainProgress::ComputingFeatures(progress_counter.clone()));
	let features = features::compute_features_dataframe(dataframe_train, &feature_groups, &|| {
		progress_counter.inc(1)
	});
	let labels = dataframe_train
		.columns
		.get(target_column_index)
		.unwrap()
		.as_number()
		.unwrap()
		.clone();
	let base: usize = 2;
	let tree_options = tree::TrainOptions {
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
	let model = tree::Regressor::train(features.view(), labels, tree_options, &mut |progress| {
		update_progress(TrainProgress::TrainingModel(ModelTrainProgress::Tree(
			progress,
		)))
	});
	TrainModelOutput::TreeRegressor(TreeRegressorTrainModelOutput {
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
	update_progress: &mut dyn FnMut(TrainProgress),
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_counter = ProgressCounter::new(dataframe_train.nrows().to_u64().unwrap());
	update_progress(TrainProgress::ComputingFeatures(progress_counter.clone()));
	let mut features = unsafe { Array2::uninitialized((dataframe_train.nrows(), n_features)) };
	features::compute_features_ndarray(
		dataframe_train,
		&feature_groups,
		features.view_mut(),
		&|| progress_counter.inc(1),
	);
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
	let model = linear::BinaryClassifier::train(
		features.view(),
		&labels,
		&linear_options,
		&mut |progress| {
			update_progress(TrainProgress::TrainingModel(ModelTrainProgress::Linear(
				progress,
			)))
		},
	);
	TrainModelOutput::LinearBinaryClassifier(LinearBinaryClassifierTrainModelOutput {
		model,
		feature_groups,
		target_column_index,
		options,
	})
}

fn train_tree_binary_classifier(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<features::FeatureGroup>,
	options: grid::TreeModelTrainOptions,
	update_progress: &mut dyn FnMut(TrainProgress),
) -> TrainModelOutput {
	let progress_counter = ProgressCounter::new(dataframe_train.nrows().to_u64().unwrap());
	update_progress(TrainProgress::ComputingFeatures(progress_counter.clone()));
	let features = features::compute_features_dataframe(dataframe_train, &feature_groups, &|| {
		progress_counter.inc(1)
	});
	let labels = dataframe_train
		.columns
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap()
		.clone();
	let base: usize = 2;
	let tree_options = tree::TrainOptions {
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
	let model =
		tree::BinaryClassifier::train(features.view(), labels, tree_options, &mut |progress| {
			update_progress(TrainProgress::TrainingModel(ModelTrainProgress::Tree(
				progress,
			)))
		});
	TrainModelOutput::TreeBinaryClassifier(TreeBinaryClassifierTrainModelOutput {
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
	update_progress: &mut dyn FnMut(TrainProgress),
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_counter = ProgressCounter::new(dataframe_train.nrows().to_u64().unwrap());
	update_progress(TrainProgress::ComputingFeatures(progress_counter.clone()));
	let mut features = unsafe { Array2::uninitialized((dataframe_train.nrows(), n_features)) };
	features::compute_features_ndarray(
		dataframe_train,
		&feature_groups,
		features.view_mut(),
		&|| progress_counter.inc(1),
	);
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
	let model = linear::MulticlassClassifier::train(
		features.view(),
		&labels,
		&linear_options,
		&mut |progress| {
			update_progress(TrainProgress::TrainingModel(ModelTrainProgress::Linear(
				progress,
			)))
		},
	);
	TrainModelOutput::LinearMulticlassClassifier(LinearMulticlassClassifierTrainModelOutput {
		model,
		feature_groups,
		target_column_index,
		options,
	})
}

fn train_tree_multiclass_classifier(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<features::FeatureGroup>,
	options: grid::TreeModelTrainOptions,
	update_progress: &mut dyn FnMut(TrainProgress),
) -> TrainModelOutput {
	let progress_counter = ProgressCounter::new(dataframe_train.nrows().to_u64().unwrap());
	update_progress(TrainProgress::ComputingFeatures(progress_counter.clone()));
	let features = features::compute_features_dataframe(dataframe_train, &feature_groups, &|| {
		progress_counter.inc(1)
	});
	let labels = dataframe_train
		.columns
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap()
		.clone();
	let base: usize = 2;
	let tree_options = tree::TrainOptions {
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
	let model =
		tree::MulticlassClassifier::train(features.view(), labels, tree_options, &mut |progress| {
			update_progress(TrainProgress::TrainingModel(ModelTrainProgress::Tree(
				progress,
			)))
		});
	TrainModelOutput::TreeMulticlassClassifier(TreeMulticlassClassifierTrainModelOutput {
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
	update_progress: &mut dyn FnMut(ModelTestProgress),
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
				update_progress,
			))
		}
		TrainModelOutput::TreeRegressor(train_model_output) => {
			let TreeRegressorTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			TestMetrics::Regression(test::test_tree_regressor(
				&dataframe_comparison,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
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
				update_progress,
			);
			TestMetrics::Classification((metrics, Some(model_metrics)))
		}
		TrainModelOutput::TreeBinaryClassifier(train_model_output) => {
			let TreeBinaryClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let (metrics, model_metrics) = test::test_tree_binary_classifier(
				&dataframe_comparison,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
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
					update_progress,
				),
				None,
			))
		}
		TrainModelOutput::TreeMulticlassClassifier(train_model_output) => {
			let TreeMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			TestMetrics::Classification((
				test::test_tree_multiclass_classifier(
					&dataframe_comparison,
					*target_column_index,
					feature_groups,
					model,
					update_progress,
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
	update_progress: &mut dyn FnMut(ModelTestProgress),
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
				update_progress,
			))
		}
		TrainModelOutput::TreeRegressor(train_model_output) => {
			let TreeRegressorTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			TestMetrics::Regression(test::test_tree_regressor(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
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
				update_progress,
			);
			TestMetrics::Classification((metrics, Some(model_metrics)))
		}
		TrainModelOutput::TreeBinaryClassifier(train_model_output) => {
			let TreeBinaryClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let (metrics, model_metrics) = test::test_tree_binary_classifier(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
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
				update_progress,
			);
			TestMetrics::Classification((metrics, None))
		}
		TrainModelOutput::TreeMulticlassClassifier(train_model_output) => {
			let TreeMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let metrics = test::test_tree_multiclass_classifier(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
			);
			TestMetrics::Classification((metrics, None))
		}
	}
}

impl Into<model::StatsSettings> for stats::StatsSettings {
	fn into(self) -> model::StatsSettings {
		model::StatsSettings {
			text_histogram_max_size: self.text_histogram_max_size.to_u64().unwrap(),
			number_histogram_max_size: self.number_histogram_max_size.to_u64().unwrap(),
		}
	}
}

impl Into<model::FeatureGroup> for features::FeatureGroup {
	fn into(self) -> model::FeatureGroup {
		match self {
			Self::Identity(f) => model::FeatureGroup::Identity(f.into()),
			Self::Normalized(f) => model::FeatureGroup::Normalized(f.into()),
			Self::OneHotEncoded(f) => model::FeatureGroup::OneHotEncoded(f.into()),
			Self::BagOfWords(f) => model::FeatureGroup::BagOfWords(f.into()),
		}
	}
}

impl Into<model::IdentityFeatureGroup> for features::IdentityFeatureGroup {
	fn into(self) -> model::IdentityFeatureGroup {
		model::IdentityFeatureGroup {
			source_column_name: self.source_column_name,
		}
	}
}

impl Into<model::NormalizedFeatureGroup> for features::NormalizedFeatureGroup {
	fn into(self) -> model::NormalizedFeatureGroup {
		model::NormalizedFeatureGroup {
			source_column_name: self.source_column_name,
			mean: self.mean,
			variance: self.variance,
		}
	}
}

impl Into<model::OneHotEncodedFeatureGroup> for features::OneHotEncodedFeatureGroup {
	fn into(self) -> model::OneHotEncodedFeatureGroup {
		model::OneHotEncodedFeatureGroup {
			source_column_name: self.source_column_name,
			categories: self.categories,
		}
	}
}

impl Into<model::BagOfWordsFeatureGroup> for features::BagOfWordsFeatureGroup {
	fn into(self) -> model::BagOfWordsFeatureGroup {
		model::BagOfWordsFeatureGroup {
			source_column_name: self.source_column_name,
			tokenizer: self.tokenizer.into(),
			tokens: self.tokens,
		}
	}
}

impl Into<model::Tokenizer> for features::Tokenizer {
	fn into(self) -> model::Tokenizer {
		match self {
			Self::Alphanumeric => model::Tokenizer::Alphanumeric,
		}
	}
}

impl Into<model::ColumnStats> for stats::ColumnStats {
	fn into(self) -> model::ColumnStats {
		match self {
			Self::Unknown(c) => model::ColumnStats::Unknown(c.into()),
			Self::Number(c) => model::ColumnStats::Number(c.into()),
			Self::Enum(c) => model::ColumnStats::Enum(c.into()),
			Self::Text(c) => model::ColumnStats::Text(c.into()),
		}
	}
}

impl Into<model::UnknownColumnStats> for stats::UnknownColumnStats {
	fn into(self) -> model::UnknownColumnStats {
		model::UnknownColumnStats {
			column_name: self.column_name,
		}
	}
}

impl Into<model::NumberColumnStats> for stats::NumberColumnStats {
	fn into(self) -> model::NumberColumnStats {
		model::NumberColumnStats {
			column_name: self.column_name,
			histogram: self.histogram,
			invalid_count: self.invalid_count,
			unique_count: self.unique_count,
			min: self.min,
			max: self.max,
			mean: self.mean,
			variance: self.variance,
			std: self.std,
			p25: self.p25,
			p50: self.p50,
			p75: self.p75,
		}
	}
}

impl Into<model::EnumColumnStats> for stats::EnumColumnStats {
	fn into(self) -> model::EnumColumnStats {
		model::EnumColumnStats {
			column_name: self.column_name,
			histogram: self
				.histogram
				.into_iter()
				.map(|(s, v)| (s, v.to_u64().unwrap()))
				.collect(),
			invalid_count: self.invalid_count.to_u64().unwrap(),
			unique_count: self.unique_count.to_u64().unwrap(),
		}
	}
}

impl Into<model::TextColumnStats> for stats::TextColumnStats {
	fn into(self) -> model::TextColumnStats {
		model::TextColumnStats {
			column_name: self.column_name,
			top_tokens: self
				.top_tokens
				.into_iter()
				.map(|(token, count, _)| (token, count))
				.collect(),
		}
	}
}

impl Into<model::RegressionMetrics> for metrics::RegressionMetricsOutput {
	fn into(self) -> model::RegressionMetrics {
		model::RegressionMetrics {
			mse: self.mse,
			rmse: self.rmse,
			mae: self.mae,
			r2: self.r2,
			baseline_mse: self.baseline_mse,
			baseline_rmse: self.baseline_rmse,
		}
	}
}

impl Into<model::ClassificationMetrics> for metrics::ClassificationMetricsOutput {
	fn into(self) -> model::ClassificationMetrics {
		model::ClassificationMetrics {
			class_metrics: self.class_metrics.into_iter().map(Into::into).collect(),
			accuracy: self.accuracy,
			precision_unweighted: self.precision_unweighted,
			precision_weighted: self.precision_weighted,
			recall_unweighted: self.recall_unweighted,
			recall_weighted: self.recall_weighted,
			baseline_accuracy: self.baseline_accuracy,
		}
	}
}

impl Into<model::ClassMetrics> for metrics::ClassMetrics {
	fn into(self) -> model::ClassMetrics {
		model::ClassMetrics {
			true_positives: self.true_positives,
			false_positives: self.false_positives,
			true_negatives: self.true_negatives,
			false_negatives: self.false_negatives,
			accuracy: self.accuracy,
			precision: self.precision,
			recall: self.recall,
			f1_score: self.f1_score,
		}
	}
}

impl Into<model::Tree> for crate::tree::Tree {
	fn into(self) -> model::Tree {
		model::Tree {
			nodes: self.nodes.into_iter().map(Into::into).collect(),
		}
	}
}

impl Into<model::Node> for crate::tree::Node {
	fn into(self) -> model::Node {
		match self {
			Self::Branch(branch) => model::Node::Branch(branch.into()),
			Self::Leaf(leaf) => model::Node::Leaf(leaf.into()),
		}
	}
}

impl Into<model::BranchNode> for crate::tree::BranchNode {
	fn into(self) -> model::BranchNode {
		model::BranchNode {
			left_child_index: self.left_child_index.to_u64().unwrap(),
			right_child_index: self.right_child_index.to_u64().unwrap(),
			split: self.split.into(),
			examples_fraction: self.examples_fraction,
		}
	}
}

impl Into<model::BranchSplit> for crate::tree::BranchSplit {
	fn into(self) -> model::BranchSplit {
		match self {
			Self::Continuous(value) => model::BranchSplit::Continuous(value.into()),
			Self::Discrete(value) => model::BranchSplit::Discrete(value.into()),
		}
	}
}

impl Into<model::BranchSplitContinuous> for crate::tree::BranchSplitContinuous {
	fn into(self) -> model::BranchSplitContinuous {
		let invalid_values_direction = match self.invalid_values_direction {
			crate::tree::SplitDirection::Left => false,
			crate::tree::SplitDirection::Right => true,
		};
		model::BranchSplitContinuous {
			feature_index: self.feature_index.to_u64().unwrap(),
			split_value: self.split_value,
			invalid_values_direction,
		}
	}
}

impl Into<model::BranchSplitDiscrete> for crate::tree::BranchSplitDiscrete {
	fn into(self) -> model::BranchSplitDiscrete {
		let directions: Vec<bool> = (0..self.directions.n)
			.map(|i| self.directions.get(i).unwrap())
			.collect();
		model::BranchSplitDiscrete {
			feature_index: self.feature_index.to_u64().unwrap(),
			directions,
		}
	}
}

impl Into<model::LeafNode> for crate::tree::LeafNode {
	fn into(self) -> model::LeafNode {
		model::LeafNode {
			value: self.value,
			examples_fraction: self.examples_fraction,
		}
	}
}

impl Into<model::ClassificationModel> for ClassificationModel {
	fn into(self) -> model::ClassificationModel {
		match self {
			Self::LinearBinary(m) => model::ClassificationModel::LinearBinary(m.into()),
			Self::LinearMulticlass(m) => model::ClassificationModel::LinearMulticlass(m.into()),
			Self::TreeBinary(m) => model::ClassificationModel::TreeBinary(m.into()),
			Self::TreeMulticlass(m) => model::ClassificationModel::TreeMulticlass(m.into()),
		}
	}
}

impl Into<model::LinearBinaryClassifier> for LinearBinaryClassifier {
	fn into(self) -> model::LinearBinaryClassifier {
		model::LinearBinaryClassifier {
			feature_groups: self.feature_groups.into_iter().map(|f| f.into()).collect(),
			options: self.options.into(),
			class_metrics: self.class_metrics.into_iter().map(Into::into).collect(),
			auc_roc: self.auc_roc,
			means: self.model.means,
			weights: self.model.weights.into_raw_vec(),
			bias: self.model.bias,
			losses: self.model.losses,
			classes: self.model.classes,
		}
	}
}

impl Into<model::LinearModelTrainOptions> for grid::LinearModelTrainOptions {
	fn into(self) -> model::LinearModelTrainOptions {
		model::LinearModelTrainOptions {
			early_stopping_fraction: self.early_stopping_fraction,
			l2_regularization: self.l2_regularization,
			learning_rate: self.learning_rate,
			max_epochs: self.max_epochs,
			n_examples_per_batch: self.n_examples_per_batch,
		}
	}
}

impl Into<model::TreeModelTrainOptions> for grid::TreeModelTrainOptions {
	fn into(self) -> model::TreeModelTrainOptions {
		model::TreeModelTrainOptions {
			depth: self.max_depth,
			learning_rate: self.learning_rate,
			min_examples_per_leaf: self.min_examples_per_leaf,
			max_rounds: self.max_rounds,
			early_stopping_fraction: self.early_stopping_fraction,
		}
	}
}

impl Into<model::LinearMulticlassClassifier> for LinearMulticlassClassifier {
	fn into(self) -> model::LinearMulticlassClassifier {
		model::LinearMulticlassClassifier {
			feature_groups: self.feature_groups.into_iter().map(|f| f.into()).collect(),
			n_features: self.model.weights.nrows().to_u64().unwrap(),
			n_classes: self.model.weights.ncols().to_u64().unwrap(),
			weights: self.model.weights.into_raw_vec(),
			biases: self.model.biases.into_raw_vec(),
			losses: self.model.losses,
			options: self.options.into(),
			classes: self.model.classes,
			means: self.model.means,
		}
	}
}

impl Into<model::TreeBinaryClassifier> for TreeBinaryClassifier {
	fn into(self) -> model::TreeBinaryClassifier {
		let losses = self.model.losses.unwrap();
		let trees = self.model.trees.into_iter().map(Into::into).collect();
		let class_metrics = self.class_metrics.into_iter().map(Into::into).collect();
		let feature_importances = self.model.feature_importances.unwrap();
		let options = self.options.into();
		model::TreeBinaryClassifier {
			feature_groups: self.feature_groups.into_iter().map(|f| f.into()).collect(),
			trees,
			class_metrics,
			bias: self.model.bias,
			losses,
			auc_roc: self.auc_roc,
			classes: self.model.classes,
			feature_importances,
			options,
		}
	}
}

impl Into<model::TreeMulticlassClassifier> for TreeMulticlassClassifier {
	fn into(self) -> model::TreeMulticlassClassifier {
		model::TreeMulticlassClassifier {
			n_rounds: self.model.n_rounds.to_u64().unwrap(),
			n_classes: self.model.n_classes.to_u64().unwrap(),
			biases: self.model.biases,
			options: self.options.into(),
			trees: self.model.trees.into_iter().map(|t| t.into()).collect(),
			feature_groups: self.feature_groups.into_iter().map(|t| t.into()).collect(),
			losses: self.model.losses.unwrap(),
			classes: self.model.classes,
			feature_importances: self.model.feature_importances.unwrap(),
		}
	}
}

impl Into<model::BinaryClassifierClassMetrics> for metrics::BinaryClassificationClassMetricsOutput {
	fn into(self) -> model::BinaryClassifierClassMetrics {
		model::BinaryClassifierClassMetrics {
			thresholds: self.thresholds.into_iter().map(Into::into).collect(),
		}
	}
}

impl Into<model::ThresholdMetrics> for metrics::BinaryClassificationThresholdMetricsOutput {
	fn into(self) -> model::ThresholdMetrics {
		model::ThresholdMetrics {
			threshold: self.threshold,
			true_positives: self.true_positives,
			false_positives: self.false_positives,
			true_negatives: self.true_negatives,
			false_negatives: self.false_negatives,
			accuracy: self.accuracy,
			precision: self.precision,
			recall: self.recall,
			f1_score: self.f1_score,
			true_positive_rate: self.true_positive_rate,
			false_positive_rate: self.false_positive_rate,
		}
	}
}

impl Into<model::ClassificationComparisonMetric> for ClassificationComparisonMetric {
	fn into(self) -> model::ClassificationComparisonMetric {
		match self {
			Self::Accuracy => model::ClassificationComparisonMetric::Accuracy,
			Self::Aucroc => model::ClassificationComparisonMetric::Aucroc,
			Self::F1 => model::ClassificationComparisonMetric::F1,
		}
	}
}

impl Into<model::RegressionModel> for RegressionModel {
	fn into(self) -> model::RegressionModel {
		match self {
			Self::Linear(m) => model::RegressionModel::Linear(m.into()),
			Self::Tree(m) => model::RegressionModel::Tree(m.into()),
		}
	}
}

impl Into<model::LinearRegressor> for LinearRegressor {
	fn into(self) -> model::LinearRegressor {
		model::LinearRegressor {
			feature_groups: self.feature_groups.into_iter().map(|f| f.into()).collect(),
			weights: self.model.weights.into_raw_vec(),
			bias: self.model.bias,
			losses: self.model.losses,
			means: self.model.means,
			options: self.options.into(),
		}
	}
}

impl Into<model::TreeRegressor> for TreeRegressor {
	fn into(self) -> model::TreeRegressor {
		let losses = self.model.losses.unwrap();
		let trees = self.model.trees.into_iter().map(Into::into).collect();
		model::TreeRegressor {
			feature_groups: self.feature_groups.into_iter().map(|f| f.into()).collect(),
			trees,
			bias: self.model.bias,
			losses,
			options: self.options.into(),
			feature_importances: self.model.feature_importances.unwrap(),
		}
	}
}

impl Into<model::RegressionComparisonMetric> for RegressionComparisonMetric {
	fn into(self) -> model::RegressionComparisonMetric {
		match self {
			Self::MeanAbsoluteError => model::RegressionComparisonMetric::MeanAbsoluteError,
			Self::MeanSquaredError => model::RegressionComparisonMetric::MeanSquaredError,
			Self::RootMeanSquaredError => model::RegressionComparisonMetric::RootMeanSquaredError,
			Self::R2 => model::RegressionComparisonMetric::R2,
		}
	}
}
