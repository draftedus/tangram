use crate::{
	config::{self, Config},
	grid, model, stats, test,
};
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::{collections::BTreeMap, path::Path};
use tangram_dataframe::prelude::*;
use tangram_metrics::StreamingMetric;
use tangram_util::{err, error::Result, id::Id, progress_counter::ProgressCounter};

/**
Train a model.
*/
pub fn train(
	model_id: Id,
	file_path: Option<&Path>,
	file_path_train: Option<&Path>,
	file_path_test: Option<&Path>,
	target_column_name: &str,
	config_path: Option<&Path>,
	update_progress: &mut dyn FnMut(Progress),
) -> Result<model::Model> {
	// Load the config from the config file, if provided.
	let config: Option<Config> = load_config(config_path)?;

	// Get the column types from the config, if set.
	let column_types: Option<BTreeMap<String, DataFrameColumnType>> = config
		.as_ref()
		.and_then(|config| config.column_types.as_ref())
		.map(|column_types| {
			column_types
				.iter()
				.map(|(column_name, column_type)| {
					let column_type = match column_type {
						config::ColumnType::Unknown => DataFrameColumnType::Unknown,
						config::ColumnType::Number => DataFrameColumnType::Number,
						config::ColumnType::Enum { options } => DataFrameColumnType::Enum {
							options: options.clone(),
						},
						config::ColumnType::Text => DataFrameColumnType::Text,
					};
					(column_name.clone(), column_type)
				})
				.collect()
		});
	// Load the dataframe from the csv file.
	let mut dataframe = if let Some(file_path) = file_path {
		Some(load_dataframe(
			file_path,
			column_types.clone(),
			update_progress,
		)?)
	} else {
		None
	};
	let dataframe_train = if let Some(file_path_train) = file_path_train {
		Some(load_dataframe(
			file_path_train,
			column_types,
			update_progress,
		)?)
	} else {
		None
	};
	let dataframe_test = if let Some(file_path_test) = file_path_test {
		// If the config has no column types set, force the column types based on the types inferred in dataframe_train.
		let column_types = dataframe_train
			.as_ref()
			.unwrap()
			.columns()
			.iter()
			.map(|column| match column {
				DataFrameColumn::Unknown(column) => (
					column.name().to_owned().unwrap(),
					DataFrameColumnType::Unknown,
				),
				DataFrameColumn::Enum(column) => (
					column.name().to_owned().unwrap(),
					DataFrameColumnType::Enum {
						options: column.options().to_owned(),
					},
				),
				DataFrameColumn::Number(column) => (
					column.name().to_owned().unwrap(),
					DataFrameColumnType::Number,
				),
				DataFrameColumn::Text(column) => {
					(column.name().to_owned().unwrap(), DataFrameColumnType::Text)
				}
			})
			.collect();
		Some(load_dataframe(
			file_path_test,
			Some(column_types),
			update_progress,
		)?)
	} else {
		None
	};

	let (dataframe_train, dataframe_test) = if let Some(dataframe) = dataframe.as_mut() {
		// Shuffle the dataframe if enabled.
		shuffle(dataframe, &config, update_progress);
		// Split the dataframe into train and test dataframes.
		let test_fraction = config
			.as_ref()
			.and_then(|config| config.test_fraction)
			.unwrap_or(0.2);
		let n_records_train = ((1.0 - test_fraction) * dataframe.nrows().to_f32().unwrap())
			.to_usize()
			.unwrap();
		let split_index = n_records_train;
		dataframe.view().split_at_row(split_index)
	} else {
		(
			dataframe_train.as_ref().unwrap().view(),
			dataframe_test.as_ref().unwrap().view(),
		)
	};

	// Retrieve the column names.
	let column_names: Vec<String> = dataframe_train
		.columns()
		.iter()
		.map(|column| column.name().unwrap().to_owned())
		.collect();

	let train_row_count = dataframe_train.nrows();
	let test_row_count = dataframe_test.nrows();

	// Compute stats.
	let stats_settings = match config
		.as_ref()
		.and_then(|config| config.text_features_max_tokens_count)
	{
		Some(max_tokens_count) => stats::StatsSettings {
			max_tokens_count,
			..Default::default()
		},
		None => stats::StatsSettings {
			..Default::default()
		},
	};
	let train_column_stats = stats::Stats::compute(&dataframe_train, &stats_settings);
	let test_column_stats = stats::Stats::compute(&dataframe_test, &stats_settings);
	let overall_column_stats = train_column_stats.clone().merge(test_column_stats.clone());
	let mut train_column_stats = train_column_stats.finalize(&stats_settings).0;
	let mut test_column_stats = test_column_stats.finalize(&stats_settings).0;
	let mut overall_column_stats = overall_column_stats.finalize(&stats_settings).0;

	// Find the target column.
	let target_column_index = column_names
		.iter()
		.position(|column_name| *column_name == target_column_name)
		.ok_or_else(|| {
			err!(
				"did not find target column \"{}\" among column names \"{}\"",
				target_column_name,
				column_names.join(", ")
			)
		})?;

	// Pull out the target column from the column stats.
	let train_target_column_stats = train_column_stats.remove(target_column_index);
	let test_target_column_stats = test_column_stats.remove(target_column_index);
	let overall_target_column_stats = overall_column_stats.remove(target_column_index);

	// Determine the task.
	let task = match &overall_target_column_stats {
		stats::ColumnStatsOutput::Number(_) => Task::Regression,
		stats::ColumnStatsOutput::Enum(target_column) => match target_column.unique_count {
			2 => Task::BinaryClassification,
			_ => Task::MulticlassClassification,
		},
		_ => return Err(err!("invalid target column type")),
	};

	// Compute the baseline metrics.
	let baseline_metrics = match task {
		Task::Regression => {
			let labels = dataframe_train.columns().get(target_column_index).unwrap();
			let labels = labels.as_number().unwrap();
			let train_target_column_stats = match &train_target_column_stats {
				stats::ColumnStatsOutput::Number(train_target_column_stats) => {
					train_target_column_stats
				}
				_ => unreachable!(),
			};
			let baseline_prediction = train_target_column_stats.mean;
			let mut metrics = tangram_metrics::RegressionMetrics::new();
			for label in labels.iter() {
				metrics.update(tangram_metrics::RegressionMetricsInput {
					predictions: &[baseline_prediction],
					labels: &[*label],
				});
			}
			Metrics::Regression(metrics.finalize())
		}
		Task::BinaryClassification => {
			let labels = dataframe_train.columns().get(target_column_index).unwrap();
			let labels = labels.as_enum().unwrap();
			let train_target_column_stats = match &train_target_column_stats {
				stats::ColumnStatsOutput::Enum(train_target_column_stats) => {
					train_target_column_stats
				}
				_ => unreachable!(),
			};
			let total_count = train_target_column_stats.count.to_f32().unwrap();
			let baseline_probability = train_target_column_stats
				.histogram
				.iter()
				.last()
				.unwrap()
				.1
				.to_f32()
				.unwrap() / total_count;
			let mut metrics = tangram_metrics::BinaryClassificationMetrics::new(3);
			for label in labels.iter() {
				metrics.update(tangram_metrics::BinaryClassificationMetricsInput {
					probabilities: &[baseline_probability],
					labels: &[*label],
				});
			}
			Metrics::BinaryClassification(metrics.finalize())
		}
		Task::MulticlassClassification => {
			let labels = dataframe_train.columns().get(target_column_index).unwrap();
			let labels = labels.as_enum().unwrap();
			let train_target_column_stats = match &train_target_column_stats {
				stats::ColumnStatsOutput::Enum(train_target_column_stats) => {
					train_target_column_stats
				}
				_ => unreachable!(),
			};
			let total_count = train_target_column_stats.count.to_f32().unwrap();
			let baseline_probabilities = train_target_column_stats
				.histogram
				.iter()
				.map(|(_, count)| count.to_f32().unwrap() / total_count)
				.collect::<Vec<_>>();
			let mut metrics = tangram_metrics::MulticlassClassificationMetrics::new(
				train_target_column_stats.histogram.len(),
			);
			for label in labels.iter() {
				metrics.update(tangram_metrics::MulticlassClassificationMetricsInput {
					probabilities: ArrayView::from(baseline_probabilities.as_slice())
						.insert_axis(Axis(0)),
					labels: ArrayView::from(&[*label]),
				});
			}
			Metrics::MulticlassClassification(metrics.finalize())
		}
	};

	// Split the train dataset into train and model comparison datasets.
	let comparison_fraction = 0.1;
	let split_index = ((1.0 - comparison_fraction) * dataframe_train.nrows().to_f32().unwrap())
		.to_usize()
		.unwrap();
	let (dataframe_train, dataframe_comparison) = dataframe_train.split_at_row(split_index);

	// Choose the comparison metric.
	let comparison_metric = choose_comparison_metric(&config, &task)?;

	// Create the hyperparameter grid.
	let grid =
		compute_hyperparameter_grid(&config, &task, target_column_index, &train_column_stats);

	// Train each model in the grid and compute model comparison metrics.
	let num_models = grid.len();
	let outputs: Vec<(TrainModelOutput, Metrics, std::time::Duration)> = grid
		.into_iter()
		.enumerate()
		.map(|(model_index, grid_item)| {
			let start = std::time::Instant::now();
			let train_model_output = train_model(grid_item, &dataframe_train, &mut |progress| {
				update_progress(Progress::Training(GridTrainProgress {
					current: model_index.to_u64().unwrap() + 1,
					total: num_models.to_u64().unwrap(),
					grid_item_progress: progress,
				}))
			});
			let duration = start.elapsed();
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
			(train_model_output, model_comparison_metrics, duration)
		})
		.collect();

	// Assemble the grid.
	let grid = compute_grid(outputs.as_slice(), &comparison_metric);

	// Choose the best model.
	let (train_model_output, best_model_index) = choose_best_model(outputs, &comparison_metric);

	// Test the best model.
	update_progress(Progress::Testing);
	let test_metrics = test_model(&train_model_output, &dataframe_test, &mut |_| {});

	// Assemble the model.
	let model = match task {
		Task::Regression => {
			let baseline_metrics = match baseline_metrics {
				Metrics::Regression(baseline_metrics) => baseline_metrics,
				_ => unreachable!(),
			};
			let comparison_metric = match comparison_metric {
				ComparisonMetric::Regression(comparison_metric) => comparison_metric,
				_ => unreachable!(),
			};
			let test_metrics = match test_metrics {
				Metrics::Regression(test_metrics) => test_metrics,
				_ => unreachable!(),
			};
			let model = match train_model_output {
				TrainModelOutput::LinearRegressor(LinearRegressorTrainModelOutput {
					model,
					feature_groups,
					train_options,
					losses,
					feature_importances,
					..
				}) => RegressionModel::Linear(LinearRegressionModel {
					model,
					train_options,
					feature_groups,
					losses,
					feature_importances,
				}),
				TrainModelOutput::TreeRegressor(TreeRegressorTrainModelOutput {
					model,
					feature_groups,
					train_options,
					losses,
					feature_importances,
					..
				}) => RegressionModel::Tree(TreeRegressionModel {
					model,
					feature_groups,
					train_options,
					losses,
					feature_importances,
				}),
				_ => unreachable!(),
			};
			model::Model::Regressor(model::Regressor {
				id: model_id.to_string(),
				target_column_name: target_column_name.to_owned(),
				test_row_count: test_row_count.to_u64().unwrap(),
				train_row_count: train_row_count.to_u64().unwrap(),
				stats_settings: stats_settings.into(),
				overall_column_stats: overall_column_stats.into_iter().map(Into::into).collect(),
				overall_target_column_stats: overall_target_column_stats.into(),
				train_column_stats: train_column_stats.into_iter().map(Into::into).collect(),
				train_target_column_stats: train_target_column_stats.into(),
				test_column_stats: test_column_stats.into_iter().map(Into::into).collect(),
				test_target_column_stats: test_target_column_stats.into(),
				test_metrics: test_metrics.into(),
				baseline_metrics: baseline_metrics.into(),
				model: model.into(),
				comparison_metric: comparison_metric.into(),
				grid,
				best_grid_item_index: best_model_index,
			})
		}
		Task::BinaryClassification => {
			let baseline_metrics = match baseline_metrics {
				Metrics::BinaryClassification(baseline_metrics) => baseline_metrics,
				_ => unreachable!(),
			};
			let comparison_metric = match comparison_metric {
				ComparisonMetric::BinaryClassification(comparison_metric) => comparison_metric,
				_ => unreachable!(),
			};
			let test_metrics = match test_metrics {
				Metrics::BinaryClassification(test_metrics) => test_metrics,
				_ => unreachable!(),
			};
			let model = match train_model_output {
				TrainModelOutput::LinearBinaryClassifier(
					LinearBinaryClassifierTrainModelOutput {
						model,
						feature_groups,
						losses,
						train_options,
						feature_importances,
						..
					},
				) => BinaryClassificationModel::Linear(LinearBinaryClassificationModel {
					model,
					feature_groups,
					losses,
					train_options,
					feature_importances,
				}),
				TrainModelOutput::TreeBinaryClassifier(TreeBinaryClassifierTrainModelOutput {
					model,
					feature_groups,
					losses,
					train_options,
					feature_importances,
					..
				}) => BinaryClassificationModel::Tree(TreeBinaryClassificationModel {
					model,
					train_options,
					feature_groups,
					losses,
					feature_importances,
				}),
				_ => unreachable!(),
			};
			let (negative_class, positive_class) = match &train_target_column_stats {
				stats::ColumnStatsOutput::Enum(train_target_column_stats) => (
					train_target_column_stats.histogram[0].0.clone(),
					train_target_column_stats.histogram[1].0.clone(),
				),
				_ => unreachable!(),
			};
			model::Model::BinaryClassifier(model::BinaryClassifier {
				id: model_id.to_string(),
				target_column_name: target_column_name.to_owned(),
				negative_class,
				positive_class,
				test_row_count: test_row_count.to_u64().unwrap(),
				train_row_count: train_row_count.to_u64().unwrap(),
				stats_settings: stats_settings.into(),
				overall_column_stats: overall_column_stats.into_iter().map(Into::into).collect(),
				overall_target_column_stats: overall_target_column_stats.into(),
				train_column_stats: train_column_stats.into_iter().map(Into::into).collect(),
				train_target_column_stats: train_target_column_stats.into(),
				test_column_stats: test_column_stats.into_iter().map(Into::into).collect(),
				test_target_column_stats: test_target_column_stats.into(),
				test_metrics: test_metrics.into(),
				baseline_metrics: baseline_metrics.into(),
				model: model.into(),
				comparison_metric: comparison_metric.into(),
				grid,
				best_grid_item_index: best_model_index,
			})
		}
		Task::MulticlassClassification { .. } => {
			let baseline_metrics = match baseline_metrics {
				Metrics::MulticlassClassification(baseline_metrics) => baseline_metrics,
				_ => unreachable!(),
			};
			let comparison_metric = match comparison_metric {
				ComparisonMetric::MulticlassClassification(comparison_metric) => comparison_metric,
				_ => unreachable!(),
			};
			let test_metrics = match test_metrics {
				Metrics::MulticlassClassification(test_metrics) => test_metrics,
				_ => unreachable!(),
			};
			let model = match train_model_output {
				TrainModelOutput::LinearMulticlassClassifier(
					LinearMulticlassClassifierTrainModelOutput {
						model,
						feature_groups,
						train_options,
						losses,
						feature_importances,
						..
					},
				) => MulticlassClassificationModel::Linear(LinearMulticlassClassificationModel {
					model,
					train_options,
					feature_groups,
					losses,
					feature_importances,
				}),
				TrainModelOutput::TreeMulticlassClassifier(
					TreeMulticlassClassifierTrainModelOutput {
						model,
						feature_groups,
						train_options,
						losses,
						feature_importances,
						..
					},
				) => MulticlassClassificationModel::Tree(TreeMulticlassClassificationModel {
					model,
					train_options,
					feature_groups,
					losses,
					feature_importances,
				}),
				_ => unreachable!(),
			};
			let classes = match &train_target_column_stats {
				stats::ColumnStatsOutput::Enum(train_target_column_stats) => {
					train_target_column_stats
						.histogram
						.iter()
						.map(|(class, _)| class.clone())
						.collect()
				}
				_ => unreachable!(),
			};
			model::Model::MulticlassClassifier(model::MulticlassClassifier {
				id: model_id.to_string(),
				target_column_name: target_column_name.to_owned(),
				classes,
				test_row_count: test_row_count.to_u64().unwrap(),
				train_row_count: train_row_count.to_u64().unwrap(),
				stats_settings: stats_settings.into(),
				overall_column_stats: overall_column_stats.into_iter().map(Into::into).collect(),
				overall_target_column_stats: overall_target_column_stats.into(),
				train_column_stats: train_column_stats.into_iter().map(Into::into).collect(),
				train_target_column_stats: train_target_column_stats.into(),
				test_column_stats: test_column_stats.into_iter().map(Into::into).collect(),
				test_target_column_stats: test_target_column_stats.into(),
				test_metrics: test_metrics.into(),
				baseline_metrics: baseline_metrics.into(),
				model: model.into(),
				comparison_metric: comparison_metric.into(),
				grid,
				best_grid_item_index: best_model_index,
			})
		}
	};
	Ok(model)
}

enum Task {
	Regression,
	BinaryClassification,
	MulticlassClassification,
}

enum BinaryClassificationComparisonMetric {
	AUCROC,
}

enum MulticlassClassificationComparisonMetric {
	Accuracy,
}

enum RegressionModel {
	Linear(LinearRegressionModel),
	Tree(TreeRegressionModel),
}

struct LinearRegressionModel {
	pub model: tangram_linear::Regressor,
	pub train_options: tangram_linear::TrainOptions,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

struct TreeRegressionModel {
	pub model: tangram_tree::Regressor,
	pub train_options: tangram_tree::TrainOptions,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

enum RegressionComparisonMetric {
	MeanAbsoluteError,
	MeanSquaredError,
	RootMeanSquaredError,
	R2,
}

enum BinaryClassificationModel {
	Linear(LinearBinaryClassificationModel),
	Tree(TreeBinaryClassificationModel),
}

struct LinearBinaryClassificationModel {
	pub model: tangram_linear::BinaryClassifier,
	pub train_options: tangram_linear::TrainOptions,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

struct TreeBinaryClassificationModel {
	pub model: tangram_tree::BinaryClassifier,
	pub train_options: tangram_tree::TrainOptions,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

enum MulticlassClassificationModel {
	Linear(LinearMulticlassClassificationModel),
	Tree(TreeMulticlassClassificationModel),
}

struct LinearMulticlassClassificationModel {
	pub model: tangram_linear::MulticlassClassifier,
	pub train_options: tangram_linear::TrainOptions,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

struct TreeMulticlassClassificationModel {
	pub model: tangram_tree::MulticlassClassifier,
	pub train_options: tangram_tree::TrainOptions,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

enum ComparisonMetric {
	Regression(RegressionComparisonMetric),
	BinaryClassification(BinaryClassificationComparisonMetric),
	MulticlassClassification(MulticlassClassificationComparisonMetric),
}

enum Metrics {
	Regression(tangram_metrics::RegressionMetricsOutput),
	BinaryClassification(tangram_metrics::BinaryClassificationMetricsOutput),
	MulticlassClassification(tangram_metrics::MulticlassClassificationMetricsOutput),
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
	ComputingHistograms(ProgressCounter),
	Finalizing(ProgressCounter),
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

#[derive(Debug)]
pub enum ModelTrainProgress {
	Linear(tangram_linear::TrainProgress),
	Tree(tangram_tree::TrainProgress),
}

#[derive(Debug)]
pub enum ModelTestProgress {
	ComputingFeatures(ProgressCounter),
	Testing,
}

fn load_config(config_path: Option<&Path>) -> Result<Option<Config>> {
	if let Some(config_path) = config_path {
		let config = std::fs::read_to_string(config_path)?;
		let config = serde_yaml::from_str(&config)?;
		Ok(Some(config))
	} else {
		Ok(None)
	}
}

fn load_dataframe(
	file_path: &Path,
	column_types: Option<BTreeMap<String, DataFrameColumnType>>,
	update_progress: &mut dyn FnMut(Progress),
) -> Result<DataFrame> {
	let len = std::fs::metadata(file_path)?.len();
	let progress_counter = ProgressCounter::new(len);
	update_progress(Progress::Loading(progress_counter.clone()));

	let dataframe = DataFrame::from_path(
		file_path,
		tangram_dataframe::FromCsvOptions {
			column_types,
			infer_options: Default::default(),
			..Default::default()
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
	// Check if shuffling is enabled in the config. If it is, use the seed from the config.
	let default_seed = 12;
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
	// Shuffle the dataframe.
	if let Some(seed) = shuffle_options {
		update_progress(Progress::Shuffling);
		dataframe.shuffle(seed)
	}
}

fn compute_hyperparameter_grid(
	config: &Option<Config>,

	task: &Task,
	target_column_index: usize,
	train_column_stats: &[stats::ColumnStatsOutput],
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
			Task::BinaryClassification => grid::compute_binary_classification_hyperparameter_grid(
				grid,
				target_column_index,
				&train_column_stats,
			),
			Task::MulticlassClassification { .. } => {
				grid::compute_multiclass_classification_hyperparameter_grid(
					grid,
					target_column_index,
					&train_column_stats,
				)
			}
		})
		.unwrap_or_else(|| match &task {
			Task::Regression => grid::default_regression_hyperparameter_grid(
				target_column_index,
				&train_column_stats,
			),
			Task::BinaryClassification => grid::default_binary_classification_hyperparameter_grid(
				target_column_index,
				&train_column_stats,
			),
			Task::MulticlassClassification { .. } => {
				grid::default_multiclass_classification_hyperparameter_grid(
					target_column_index,
					&train_column_stats,
				)
			}
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
	model: tangram_linear::Regressor,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	target_column_index: usize,
	losses: Option<Vec<f32>>,
	train_options: tangram_linear::TrainOptions,
	feature_importances: Vec<f32>,
}

struct TreeRegressorTrainModelOutput {
	model: tangram_tree::Regressor,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	target_column_index: usize,
	losses: Option<Vec<f32>>,
	train_options: tangram_tree::TrainOptions,
	feature_importances: Vec<f32>,
}

struct LinearBinaryClassifierTrainModelOutput {
	model: tangram_linear::BinaryClassifier,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	target_column_index: usize,
	losses: Option<Vec<f32>>,
	train_options: tangram_linear::TrainOptions,
	feature_importances: Vec<f32>,
}

struct TreeBinaryClassifierTrainModelOutput {
	model: tangram_tree::BinaryClassifier,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	target_column_index: usize,
	losses: Option<Vec<f32>>,
	train_options: tangram_tree::TrainOptions,
	feature_importances: Vec<f32>,
}

struct LinearMulticlassClassifierTrainModelOutput {
	model: tangram_linear::MulticlassClassifier,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	target_column_index: usize,
	losses: Option<Vec<f32>>,
	train_options: tangram_linear::TrainOptions,
	feature_importances: Vec<f32>,
}

struct TreeMulticlassClassifierTrainModelOutput {
	model: tangram_tree::MulticlassClassifier,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	target_column_index: usize,
	losses: Option<Vec<f32>>,
	train_options: tangram_tree::TrainOptions,
	feature_importances: Vec<f32>,
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
	feature_groups: Vec<tangram_features::FeatureGroup>,
	options: grid::LinearModelTrainOptions,
	update_progress: &mut dyn FnMut(TrainProgress),
) -> TrainModelOutput {
	let progress_counter = ProgressCounter::new(dataframe_train.nrows().to_u64().unwrap());
	update_progress(TrainProgress::ComputingFeatures(progress_counter.clone()));
	let features =
		tangram_features::compute_features_array_f32(dataframe_train, &feature_groups, &|| {
			progress_counter.inc(1)
		});
	let labels = dataframe_train
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_number()
		.unwrap();
	let linear_options = compute_linear_options(&options);
	let progress = &mut |progress| {
		update_progress(TrainProgress::TrainingModel(ModelTrainProgress::Linear(
			progress,
		)))
	};
	let train_output =
		tangram_linear::Regressor::train(features.view(), labels, &linear_options, progress);
	TrainModelOutput::LinearRegressor(LinearRegressorTrainModelOutput {
		model: train_output.model,
		feature_groups,
		target_column_index,
		train_options: linear_options,
		losses: train_output.losses,
		feature_importances: train_output.feature_importances.unwrap(),
	})
}

fn train_tree_regressor(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	options: grid::TreeModelTrainOptions,
	update_progress: &mut dyn FnMut(TrainProgress),
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|f| f.n_features()).sum::<usize>();
	let n_features = n_features.to_u64().unwrap();
	let n_examples = dataframe_train.nrows().to_u64().unwrap();
	let progress_counter = ProgressCounter::new(n_features * n_examples);
	update_progress(TrainProgress::ComputingFeatures(progress_counter.clone()));
	let features =
		tangram_features::compute_features_dataframe(dataframe_train, &feature_groups, &|i| {
			progress_counter.inc(i)
		});
	let labels = dataframe_train
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_number()
		.unwrap()
		.clone();
	let tree_options = compute_tree_options(&options);
	let progress = &mut |progress| {
		update_progress(TrainProgress::TrainingModel(ModelTrainProgress::Tree(
			progress,
		)))
	};
	let train_output =
		tangram_tree::Regressor::train(features.view(), labels, &tree_options, progress);
	TrainModelOutput::TreeRegressor(TreeRegressorTrainModelOutput {
		model: train_output.model,
		feature_groups,
		target_column_index,
		train_options: tree_options,
		losses: train_output.losses,
		feature_importances: train_output.feature_importances.unwrap(),
	})
}

fn train_linear_binary_classifier(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	options: grid::LinearModelTrainOptions,
	update_progress: &mut dyn FnMut(TrainProgress),
) -> TrainModelOutput {
	let progress_counter = ProgressCounter::new(dataframe_train.nrows().to_u64().unwrap());
	update_progress(TrainProgress::ComputingFeatures(progress_counter.clone()));
	let features =
		tangram_features::compute_features_array_f32(dataframe_train, &feature_groups, &|| {
			progress_counter.inc(1)
		});
	let labels = dataframe_train
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let linear_options = compute_linear_options(&options);
	let progress = &mut |progress| {
		update_progress(TrainProgress::TrainingModel(ModelTrainProgress::Linear(
			progress,
		)))
	};
	let train_output =
		tangram_linear::BinaryClassifier::train(features.view(), labels, &linear_options, progress);
	TrainModelOutput::LinearBinaryClassifier(LinearBinaryClassifierTrainModelOutput {
		model: train_output.model,
		feature_groups,
		target_column_index,
		train_options: linear_options,
		losses: train_output.losses,
		feature_importances: train_output.feature_importances.unwrap(),
	})
}

fn train_tree_binary_classifier(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	options: grid::TreeModelTrainOptions,
	update_progress: &mut dyn FnMut(TrainProgress),
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|f| f.n_features()).sum::<usize>();
	let n_features = n_features.to_u64().unwrap();
	let n_examples = dataframe_train.nrows().to_u64().unwrap();
	let progress_counter = ProgressCounter::new(n_features * n_examples);
	update_progress(TrainProgress::ComputingFeatures(progress_counter.clone()));
	let features =
		tangram_features::compute_features_dataframe(dataframe_train, &feature_groups, &|i| {
			progress_counter.inc(i)
		});
	let labels = dataframe_train
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap()
		.clone();
	let tree_options = compute_tree_options(&options);
	let progress = &mut |progress| {
		update_progress(TrainProgress::TrainingModel(ModelTrainProgress::Tree(
			progress,
		)))
	};
	let train_output =
		tangram_tree::BinaryClassifier::train(features.view(), labels, &tree_options, progress);
	TrainModelOutput::TreeBinaryClassifier(TreeBinaryClassifierTrainModelOutput {
		model: train_output.model,
		feature_groups,
		target_column_index,
		train_options: tree_options,
		losses: train_output.losses,
		feature_importances: train_output.feature_importances.unwrap(),
	})
}

fn train_linear_multiclass_classifier(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	options: grid::LinearModelTrainOptions,
	update_progress: &mut dyn FnMut(TrainProgress),
) -> TrainModelOutput {
	let progress_counter = ProgressCounter::new(dataframe_train.nrows().to_u64().unwrap());
	update_progress(TrainProgress::ComputingFeatures(progress_counter.clone()));
	let features =
		tangram_features::compute_features_array_f32(dataframe_train, &feature_groups, &|| {
			progress_counter.inc(1)
		});
	let labels = dataframe_train
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let linear_options = compute_linear_options(&options);
	let progress = &mut |progress| {
		update_progress(TrainProgress::TrainingModel(ModelTrainProgress::Linear(
			progress,
		)))
	};
	let train_output = tangram_linear::MulticlassClassifier::train(
		features.view(),
		labels,
		&linear_options,
		progress,
	);
	TrainModelOutput::LinearMulticlassClassifier(LinearMulticlassClassifierTrainModelOutput {
		model: train_output.model,
		feature_groups,
		target_column_index,
		train_options: linear_options,
		losses: train_output.losses,
		feature_importances: train_output.feature_importances.unwrap(),
	})
}

fn train_tree_multiclass_classifier(
	dataframe_train: &DataFrameView,
	target_column_index: usize,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	options: grid::TreeModelTrainOptions,
	update_progress: &mut dyn FnMut(TrainProgress),
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|f| f.n_features()).sum::<usize>();
	let n_features = n_features.to_u64().unwrap();
	let n_examples = dataframe_train.nrows().to_u64().unwrap();
	let progress_counter = ProgressCounter::new(n_features * n_examples);
	update_progress(TrainProgress::ComputingFeatures(progress_counter.clone()));
	let features =
		tangram_features::compute_features_dataframe(dataframe_train, &feature_groups, &|i| {
			progress_counter.inc(i)
		});
	let labels = dataframe_train
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap()
		.clone();
	let tree_options = compute_tree_options(&options);
	let train_output = tangram_tree::MulticlassClassifier::train(
		features.view(),
		labels,
		&tree_options,
		&mut |progress| {
			update_progress(TrainProgress::TrainingModel(ModelTrainProgress::Tree(
				progress,
			)))
		},
	);
	TrainModelOutput::TreeMulticlassClassifier(TreeMulticlassClassifierTrainModelOutput {
		model: train_output.model,
		feature_groups,
		target_column_index,
		train_options: tree_options,
		losses: train_output.losses,
		feature_importances: train_output.feature_importances.unwrap(),
	})
}

fn compute_linear_options(options: &grid::LinearModelTrainOptions) -> tangram_linear::TrainOptions {
	let mut linear_options = tangram_linear::TrainOptions::default();
	linear_options.compute_losses = true;
	if let Some(l2_regularization) = options.l2_regularization {
		linear_options.l2_regularization = l2_regularization;
	}
	if let Some(learning_rate) = options.learning_rate {
		linear_options.learning_rate = learning_rate;
	}
	if let Some(max_epochs) = options.max_epochs {
		linear_options.max_epochs = max_epochs.to_usize().unwrap();
	}
	if let Some(n_examples_per_batch) = options.n_examples_per_batch {
		linear_options.n_examples_per_batch = n_examples_per_batch.to_usize().unwrap();
	}
	if let Some(early_stopping_options) = options.early_stopping_options.as_ref() {
		linear_options.early_stopping_options = Some(tangram_linear::EarlyStoppingOptions {
			early_stopping_fraction: early_stopping_options.early_stopping_fraction,
			min_decrease_in_loss_for_significant_change: early_stopping_options
				.early_stopping_fraction,
			n_epochs_without_improvement_to_stop: early_stopping_options.early_stopping_rounds,
		})
	}
	linear_options
}

fn compute_tree_options(options: &grid::TreeModelTrainOptions) -> tangram_tree::TrainOptions {
	let mut tree_options = tangram_tree::TrainOptions::default();
	tree_options.compute_losses = true;
	if let Some(early_stopping_options) = options.early_stopping_options.as_ref() {
		tree_options.early_stopping_options = Some(tangram_tree::EarlyStoppingOptions {
			early_stopping_fraction: early_stopping_options.early_stopping_fraction,
			n_epochs_without_improvement_to_stop: early_stopping_options.early_stopping_rounds,
			min_decrease_in_loss_for_significant_change: early_stopping_options
				.early_stopping_threshold,
		})
	}
	if let Some(l2_regularization) = options.l2_regularization {
		tree_options.l2_regularization = l2_regularization;
	}
	if let Some(learning_rate) = options.learning_rate {
		tree_options.learning_rate = learning_rate;
	}
	if let Some(max_depth) = options.max_depth {
		tree_options.max_depth = Some(max_depth.to_usize().unwrap());
	}
	if let Some(max_examples_for_computing_bin_thresholds) =
		options.max_examples_for_computing_bin_thresholds
	{
		tree_options.max_examples_for_computing_bin_thresholds =
			max_examples_for_computing_bin_thresholds
				.to_usize()
				.unwrap();
	}
	if let Some(max_leaf_nodes) = options.max_leaf_nodes {
		tree_options.max_leaf_nodes = max_leaf_nodes.to_usize().unwrap();
	}
	if let Some(max_rounds) = options.max_rounds {
		tree_options.max_rounds = max_rounds.to_usize().unwrap();
	}
	if let Some(max_valid_bins_for_number_features) = options.max_valid_bins_for_number_features {
		tree_options.max_valid_bins_for_number_features = max_valid_bins_for_number_features;
	}
	if let Some(min_examples_per_node) = options.min_examples_per_node {
		tree_options.min_examples_per_node = min_examples_per_node.to_usize().unwrap();
	}
	if let Some(min_gain_to_split) = options.min_gain_to_split {
		tree_options.min_gain_to_split = min_gain_to_split;
	}
	if let Some(min_sum_hessians_per_node) = options.min_sum_hessians_per_node {
		tree_options.min_sum_hessians_per_node = min_sum_hessians_per_node;
	}
	if let Some(smoothing_factor_for_discrete_bin_sorting) =
		options.smoothing_factor_for_discrete_bin_sorting
	{
		tree_options.smoothing_factor_for_discrete_bin_sorting =
			smoothing_factor_for_discrete_bin_sorting;
	}
	if let Some(supplemental_l2_regularization_for_discrete_splits) =
		options.supplemental_l2_regularization_for_discrete_splits
	{
		tree_options.supplemental_l2_regularization_for_discrete_splits =
			supplemental_l2_regularization_for_discrete_splits;
	}
	tree_options
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
					metric => Err(err!(
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
		Task::BinaryClassification => {
			if let Some(metric) = config
				.as_ref()
				.and_then(|config| config.comparison_metric.as_ref())
			{
				match metric {
					config::ComparisonMetric::Accuracy => {
						Ok(ComparisonMetric::BinaryClassification(
							BinaryClassificationComparisonMetric::AUCROC,
						))
					}
					metric => Err(err!(
						"{} is an invalid model comparison metric for binary classification",
						metric,
					)),
				}
			} else {
				Ok(ComparisonMetric::BinaryClassification(
					BinaryClassificationComparisonMetric::AUCROC,
				))
			}
		}
		Task::MulticlassClassification { .. } => {
			if let Some(metric) = config
				.as_ref()
				.and_then(|config| config.comparison_metric.as_ref())
			{
				match metric {
					config::ComparisonMetric::Accuracy => {
						Ok(ComparisonMetric::MulticlassClassification(
							MulticlassClassificationComparisonMetric::Accuracy,
						))
					}
					metric => Err(err!(
						"{} is an invalid model comparison metric for multiclass classification",
						metric,
					)),
				}
			} else {
				Ok(ComparisonMetric::MulticlassClassification(
					MulticlassClassificationComparisonMetric::Accuracy,
				))
			}
		}
	}
}

fn compute_model_comparison_metrics(
	train_model_output: &TrainModelOutput,
	dataframe_comparison: &DataFrameView,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> Metrics {
	match train_model_output {
		TrainModelOutput::LinearRegressor(train_model_output) => {
			let LinearRegressorTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			Metrics::Regression(test::test_linear_regressor(
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
			Metrics::Regression(test::test_tree_regressor(
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
			let metrics = test::test_linear_binary_classifier(
				&dataframe_comparison,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
			);
			Metrics::BinaryClassification(metrics)
		}
		TrainModelOutput::TreeBinaryClassifier(train_model_output) => {
			let TreeBinaryClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let metrics = test::test_tree_binary_classifier(
				&dataframe_comparison,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
			);
			Metrics::BinaryClassification(metrics)
		}
		TrainModelOutput::LinearMulticlassClassifier(train_model_output) => {
			let LinearMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			Metrics::MulticlassClassification(test::test_linear_multiclass_classifier(
				&dataframe_comparison,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
			))
		}
		TrainModelOutput::TreeMulticlassClassifier(train_model_output) => {
			let TreeMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			Metrics::MulticlassClassification(test::test_tree_multiclass_classifier(
				&dataframe_comparison,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
			))
		}
	}
}

fn compute_grid(
	outputs: &[(TrainModelOutput, Metrics, std::time::Duration)],
	comparison_metric: &ComparisonMetric,
) -> Vec<model::GridItem> {
	outputs
		.iter()
		.map(|(output, metrics, duration)| {
			let model_comparison_metric_value =
				get_model_comparison_metric_value(comparison_metric, metrics);
			match output {
				TrainModelOutput::LinearRegressor(model) => {
					model::GridItem::Linear(model::LinearGridItem {
						hyperparameters: model.train_options.clone().into(),
						model_comparison_metric_value,
						duration: duration.as_secs_f32(),
					})
				}
				TrainModelOutput::TreeRegressor(model) => {
					model::GridItem::Tree(model::TreeGridItem {
						hyperparameters: model.train_options.clone().into(),
						model_comparison_metric_value,
						duration: duration.as_secs_f32(),
					})
				}
				TrainModelOutput::LinearBinaryClassifier(model) => {
					model::GridItem::Linear(model::LinearGridItem {
						hyperparameters: model.train_options.clone().into(),
						model_comparison_metric_value,
						duration: duration.as_secs_f32(),
					})
				}
				TrainModelOutput::TreeBinaryClassifier(model) => {
					model::GridItem::Tree(model::TreeGridItem {
						hyperparameters: model.train_options.clone().into(),
						model_comparison_metric_value,
						duration: duration.as_secs_f32(),
					})
				}
				TrainModelOutput::LinearMulticlassClassifier(model) => {
					model::GridItem::Linear(model::LinearGridItem {
						hyperparameters: model.train_options.clone().into(),
						model_comparison_metric_value,
						duration: duration.as_secs_f32(),
					})
				}
				TrainModelOutput::TreeMulticlassClassifier(model) => {
					model::GridItem::Tree(model::TreeGridItem {
						hyperparameters: model.train_options.clone().into(),
						model_comparison_metric_value,
						duration: duration.as_secs_f32(),
					})
				}
			}
		})
		.collect::<Vec<_>>()
}

fn get_model_comparison_metric_value(
	comparison_metric: &ComparisonMetric,
	metrics: &Metrics,
) -> f32 {
	match (comparison_metric, metrics) {
		(ComparisonMetric::Regression(comparison_metric), Metrics::Regression(metrics)) => {
			match comparison_metric {
				RegressionComparisonMetric::MeanAbsoluteError => metrics.mae,
				RegressionComparisonMetric::MeanSquaredError => metrics.mse,
				RegressionComparisonMetric::RootMeanSquaredError => metrics.rmse,
				RegressionComparisonMetric::R2 => metrics.r2,
			}
		}
		(
			ComparisonMetric::BinaryClassification(comparison_metric),
			Metrics::BinaryClassification(metrics),
		) => match comparison_metric {
			BinaryClassificationComparisonMetric::AUCROC => metrics.auc_roc_approx,
		},
		(
			ComparisonMetric::MulticlassClassification(comparison_metric),
			Metrics::MulticlassClassification(metrics),
		) => match comparison_metric {
			MulticlassClassificationComparisonMetric::Accuracy => metrics.accuracy,
		},
		_ => unreachable!(),
	}
}

fn choose_best_model(
	outputs: Vec<(TrainModelOutput, Metrics, std::time::Duration)>,
	comparison_metric: &ComparisonMetric,
) -> (TrainModelOutput, usize) {
	match comparison_metric {
		ComparisonMetric::Regression(comparison_metric) => {
			choose_best_model_regression(outputs, comparison_metric)
		}
		ComparisonMetric::BinaryClassification(comparison_metric) => {
			choose_best_model_binary_classification(outputs, comparison_metric)
		}
		ComparisonMetric::MulticlassClassification(comparison_metric) => {
			choose_best_model_multiclass_classification(outputs, comparison_metric)
		}
	}
}

fn choose_best_model_regression(
	outputs: Vec<(TrainModelOutput, Metrics, std::time::Duration)>,
	comparison_metric: &RegressionComparisonMetric,
) -> (TrainModelOutput, usize) {
	outputs
		.into_iter()
		.enumerate()
		.max_by(|(_, (_, metrics_a, _)), (_, (_, metrics_b, _))| {
			let metrics_a = match metrics_a {
				Metrics::Regression(metrics) => metrics,
				_ => unreachable!(),
			};
			let metrics_b = match metrics_b {
				Metrics::Regression(metrics) => metrics,
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
		.map(|(index, (model, _, _))| (model, index))
		.unwrap()
}

fn choose_best_model_binary_classification(
	outputs: Vec<(TrainModelOutput, Metrics, std::time::Duration)>,
	comparison_metric: &BinaryClassificationComparisonMetric,
) -> (TrainModelOutput, usize) {
	outputs
		.into_iter()
		.enumerate()
		.max_by(|(_, (_, metrics_a, _)), (_, (_, metrics_b, _))| {
			let task_metrics_a = match metrics_a {
				Metrics::BinaryClassification(metrics) => metrics,
				_ => unreachable!(),
			};
			let task_metrics_b = match metrics_b {
				Metrics::BinaryClassification(metrics) => metrics,
				_ => unreachable!(),
			};
			match comparison_metric {
				BinaryClassificationComparisonMetric::AUCROC => task_metrics_a
					.auc_roc_approx
					.partial_cmp(&task_metrics_b.auc_roc_approx)
					.unwrap(),
			}
		})
		.map(|(index, (model, _, _))| (model, index))
		.unwrap()
}

fn choose_best_model_multiclass_classification(
	outputs: Vec<(TrainModelOutput, Metrics, std::time::Duration)>,
	comparison_metric: &MulticlassClassificationComparisonMetric,
) -> (TrainModelOutput, usize) {
	outputs
		.into_iter()
		.enumerate()
		.max_by(|(_, (_, metrics_a, _)), (_, (_, metrics_b, _))| {
			let task_metrics_a = match metrics_a {
				Metrics::MulticlassClassification(metrics) => metrics,
				_ => unreachable!(),
			};
			let task_metrics_b = match metrics_b {
				Metrics::MulticlassClassification(metrics) => metrics,
				_ => unreachable!(),
			};
			match comparison_metric {
				MulticlassClassificationComparisonMetric::Accuracy => task_metrics_a
					.accuracy
					.partial_cmp(&task_metrics_b.accuracy)
					.unwrap(),
			}
		})
		.map(|(index, (model, _, _))| (model, index))
		.unwrap()
}

fn test_model(
	train_model_output: &TrainModelOutput,
	dataframe_test: &DataFrameView,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> Metrics {
	match train_model_output {
		TrainModelOutput::LinearRegressor(train_model_output) => {
			let LinearRegressorTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let test_metrics = test::test_linear_regressor(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
			);
			Metrics::Regression(test_metrics)
		}
		TrainModelOutput::TreeRegressor(train_model_output) => {
			let TreeRegressorTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let test_metrics = test::test_tree_regressor(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
			);
			Metrics::Regression(test_metrics)
		}
		TrainModelOutput::LinearBinaryClassifier(train_model_output) => {
			let LinearBinaryClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let test_metrics = test::test_linear_binary_classifier(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
			);
			Metrics::BinaryClassification(test_metrics)
		}
		TrainModelOutput::TreeBinaryClassifier(train_model_output) => {
			let TreeBinaryClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let test_metrics = test::test_tree_binary_classifier(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
			);
			Metrics::BinaryClassification(test_metrics)
		}
		TrainModelOutput::LinearMulticlassClassifier(train_model_output) => {
			let LinearMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let test_metrics = test::test_linear_multiclass_classifier(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
			);
			Metrics::MulticlassClassification(test_metrics)
		}
		TrainModelOutput::TreeMulticlassClassifier(train_model_output) => {
			let TreeMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let test_metrics = test::test_tree_multiclass_classifier(
				&dataframe_test,
				*target_column_index,
				feature_groups,
				model,
				update_progress,
			);
			Metrics::MulticlassClassification(test_metrics)
		}
	}
}

impl Into<model::StatsSettings> for stats::StatsSettings {
	fn into(self) -> model::StatsSettings {
		model::StatsSettings {
			number_histogram_max_size: self.number_histogram_max_size,
		}
	}
}

impl Into<model::FeatureGroup> for tangram_features::FeatureGroup {
	fn into(self) -> model::FeatureGroup {
		match self {
			tangram_features::FeatureGroup::Identity(feature_group) => {
				model::FeatureGroup::Identity(feature_group.into())
			}
			tangram_features::FeatureGroup::Normalized(feature_group) => {
				model::FeatureGroup::Normalized(feature_group.into())
			}
			tangram_features::FeatureGroup::OneHotEncoded(feature_group) => {
				model::FeatureGroup::OneHotEncoded(feature_group.into())
			}
			tangram_features::FeatureGroup::BagOfWords(feature_group) => {
				model::FeatureGroup::BagOfWords(feature_group.into())
			}
		}
	}
}

impl Into<model::IdentityFeatureGroup> for tangram_features::IdentityFeatureGroup {
	fn into(self) -> model::IdentityFeatureGroup {
		model::IdentityFeatureGroup {
			source_column_name: self.source_column_name,
		}
	}
}

impl Into<model::NormalizedFeatureGroup> for tangram_features::NormalizedFeatureGroup {
	fn into(self) -> model::NormalizedFeatureGroup {
		model::NormalizedFeatureGroup {
			source_column_name: self.source_column_name,
			mean: self.mean,
			variance: self.variance,
		}
	}
}

impl Into<model::OneHotEncodedFeatureGroup> for tangram_features::OneHotEncodedFeatureGroup {
	fn into(self) -> model::OneHotEncodedFeatureGroup {
		model::OneHotEncodedFeatureGroup {
			source_column_name: self.source_column_name,
			options: self.options,
		}
	}
}

impl Into<model::BagOfWordsFeatureGroup> for tangram_features::BagOfWordsFeatureGroup {
	fn into(self) -> model::BagOfWordsFeatureGroup {
		model::BagOfWordsFeatureGroup {
			source_column_name: self.source_column_name,
			tokenizer: self.tokenizer.into(),
			tokens: self
				.tokens
				.into_iter()
				.map(|token| model::BagOfWordsFeatureGroupToken {
					token: token.token.into(),
					idf: token.idf,
				})
				.collect(),
		}
	}
}

impl Into<model::Token> for tangram_features::BagOfWordsFeatureGroupToken {
	fn into(self) -> model::Token {
		match self {
			Self::Unigram(token) => model::Token::Unigram(token),
			Self::Bigram(token_a, token_b) => model::Token::Bigram(token_a, token_b),
		}
	}
}

impl Into<model::Tokenizer> for tangram_features::BagOfWordsFeatureGroupTokenizer {
	fn into(self) -> model::Tokenizer {
		match self {
			tangram_features::BagOfWordsFeatureGroupTokenizer::Alphanumeric => {
				model::Tokenizer::Alphanumeric
			}
		}
	}
}

impl Into<model::ColumnStats> for stats::ColumnStatsOutput {
	fn into(self) -> model::ColumnStats {
		match self {
			stats::ColumnStatsOutput::Unknown(column_stats) => {
				model::ColumnStats::Unknown(column_stats.into())
			}
			stats::ColumnStatsOutput::Number(column_stats) => {
				model::ColumnStats::Number(column_stats.into())
			}
			stats::ColumnStatsOutput::Enum(column_stats) => {
				model::ColumnStats::Enum(column_stats.into())
			}
			stats::ColumnStatsOutput::Text(column_stats) => {
				model::ColumnStats::Text(column_stats.into())
			}
		}
	}
}

impl Into<model::UnknownColumnStats> for stats::UnknownColumnStatsOutput {
	fn into(self) -> model::UnknownColumnStats {
		model::UnknownColumnStats {
			column_name: self.column_name,
		}
	}
}

impl Into<model::NumberColumnStats> for stats::NumberColumnStatsOutput {
	fn into(self) -> model::NumberColumnStats {
		model::NumberColumnStats {
			column_name: self.column_name,
			histogram: self.histogram.map(|histogram| {
				histogram
					.into_iter()
					.map(|(k, v)| (k.get(), v.to_u64().unwrap()))
					.collect()
			}),
			invalid_count: self.invalid_count.to_u64().unwrap(),
			unique_count: self.unique_count.to_u64().unwrap(),
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

impl Into<model::EnumColumnStats> for stats::EnumColumnStatsOutput {
	fn into(self) -> model::EnumColumnStats {
		model::EnumColumnStats {
			column_name: self.column_name,
			histogram: self
				.histogram
				.into_iter()
				.map(|(k, v)| (k, v.to_u64().unwrap()))
				.collect(),
			invalid_count: self.invalid_count.to_u64().unwrap(),
			unique_count: self.unique_count.to_u64().unwrap(),
		}
	}
}

impl Into<model::TextColumnStats> for stats::TextColumnStatsOutput {
	fn into(self) -> model::TextColumnStats {
		model::TextColumnStats {
			column_name: self.column_name,
			top_tokens: self.top_tokens.into_iter().map(Into::into).collect(),
			tokenizer: self.tokenizer.into(),
		}
	}
}

impl Into<model::Tokenizer> for stats::Tokenizer {
	fn into(self) -> model::Tokenizer {
		match self {
			stats::Tokenizer::Alphanumeric => model::Tokenizer::Alphanumeric,
		}
	}
}

impl Into<model::TokenStats> for stats::TokenStats {
	fn into(self) -> model::TokenStats {
		model::TokenStats {
			token: self.token.into(),
			occurrence_count: self.count.to_u64().unwrap(),
			examples_count: self.examples_count.to_u64().unwrap(),
		}
	}
}

impl Into<model::Token> for stats::Token {
	fn into(self) -> model::Token {
		match self {
			stats::Token::Unigram(token) => model::Token::Unigram(token),
			stats::Token::Bigram(token_a, token_b) => model::Token::Bigram(token_a, token_b),
		}
	}
}

impl Into<model::RegressionMetrics> for tangram_metrics::RegressionMetricsOutput {
	fn into(self) -> model::RegressionMetrics {
		model::RegressionMetrics {
			mse: self.mse,
			rmse: self.rmse,
			mae: self.mae,
			r2: self.r2,
		}
	}
}

impl Into<model::BinaryClassificationMetrics>
	for tangram_metrics::BinaryClassificationMetricsOutput
{
	fn into(self) -> model::BinaryClassificationMetrics {
		model::BinaryClassificationMetrics {
			auc_roc: self.auc_roc_approx,
			thresholds: self.thresholds.into_iter().map(Into::into).collect(),
		}
	}
}

impl Into<model::BinaryClassificationMetricsForThreshold>
	for tangram_metrics::BinaryClassificationMetricsOutputForThreshold
{
	fn into(self) -> model::BinaryClassificationMetricsForThreshold {
		model::BinaryClassificationMetricsForThreshold {
			threshold: self.threshold,
			true_positives: self.true_negatives,
			false_positives: self.false_negatives,
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

impl Into<model::MulticlassClassificationMetrics>
	for tangram_metrics::MulticlassClassificationMetricsOutput
{
	fn into(self) -> model::MulticlassClassificationMetrics {
		model::MulticlassClassificationMetrics {
			class_metrics: self.class_metrics.into_iter().map(Into::into).collect(),
			accuracy: self.accuracy,
			precision_unweighted: self.precision_unweighted,
			precision_weighted: self.precision_weighted,
			recall_unweighted: self.recall_unweighted,
			recall_weighted: self.recall_weighted,
		}
	}
}

impl Into<model::ClassMetrics> for tangram_metrics::ClassMetrics {
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

impl Into<model::Tree> for tangram_tree::Tree {
	fn into(self) -> model::Tree {
		model::Tree {
			nodes: self.nodes.into_iter().map(Into::into).collect(),
		}
	}
}

impl Into<model::Node> for tangram_tree::Node {
	fn into(self) -> model::Node {
		match self {
			tangram_tree::Node::Branch(branch) => model::Node::Branch(branch.into()),
			tangram_tree::Node::Leaf(leaf) => model::Node::Leaf(leaf.into()),
		}
	}
}

impl Into<model::BranchNode> for tangram_tree::BranchNode {
	fn into(self) -> model::BranchNode {
		model::BranchNode {
			left_child_index: self.left_child_index,
			right_child_index: self.right_child_index,
			split: self.split.into(),
			examples_fraction: self.examples_fraction,
		}
	}
}

impl Into<model::BranchSplit> for tangram_tree::BranchSplit {
	fn into(self) -> model::BranchSplit {
		match self {
			tangram_tree::BranchSplit::Continuous(value) => {
				model::BranchSplit::Continuous(value.into())
			}
			tangram_tree::BranchSplit::Discrete(value) => {
				model::BranchSplit::Discrete(value.into())
			}
		}
	}
}

impl Into<model::BranchSplitContinuous> for tangram_tree::BranchSplitContinuous {
	fn into(self) -> model::BranchSplitContinuous {
		let invalid_values_direction = match self.invalid_values_direction {
			tangram_tree::SplitDirection::Left => false,
			tangram_tree::SplitDirection::Right => true,
		};
		model::BranchSplitContinuous {
			feature_index: self.feature_index,
			split_value: self.split_value,
			invalid_values_direction,
		}
	}
}

impl Into<model::BranchSplitDiscrete> for tangram_tree::BranchSplitDiscrete {
	fn into(self) -> model::BranchSplitDiscrete {
		model::BranchSplitDiscrete {
			feature_index: self.feature_index,
			directions: self.directions.into_iter().map(Into::into).collect(),
		}
	}
}

impl Into<model::SplitDirection> for tangram_tree::SplitDirection {
	fn into(self) -> model::SplitDirection {
		match self {
			tangram_tree::SplitDirection::Left => model::SplitDirection::Left,
			tangram_tree::SplitDirection::Right => model::SplitDirection::Right,
		}
	}
}

impl Into<model::LeafNode> for tangram_tree::LeafNode {
	fn into(self) -> model::LeafNode {
		model::LeafNode {
			value: self.value,
			examples_fraction: self.examples_fraction,
		}
	}
}

impl Into<model::RegressionModel> for RegressionModel {
	fn into(self) -> model::RegressionModel {
		match self {
			RegressionModel::Linear(model) => model::RegressionModel::Linear(model.into()),
			RegressionModel::Tree(model) => model::RegressionModel::Tree(model.into()),
		}
	}
}

impl Into<model::LinearRegressor> for LinearRegressionModel {
	fn into(self) -> model::LinearRegressor {
		model::LinearRegressor {
			bias: self.model.bias,
			weights: self.model.weights.into_raw_vec(),
			means: self.model.means,
			train_options: self.train_options.into(),
			feature_groups: self.feature_groups.into_iter().map(Into::into).collect(),
			losses: self.losses,
			feature_importances: self.feature_importances,
		}
	}
}

impl Into<model::TreeRegressor> for TreeRegressionModel {
	fn into(self) -> model::TreeRegressor {
		model::TreeRegressor {
			bias: self.model.bias,
			trees: self.model.trees.into_iter().map(Into::into).collect(),
			train_options: self.train_options.into(),
			feature_groups: self.feature_groups.into_iter().map(Into::into).collect(),
			losses: self.losses,
			feature_importances: self.feature_importances,
		}
	}
}

impl Into<model::BinaryClassificationModel> for BinaryClassificationModel {
	fn into(self) -> model::BinaryClassificationModel {
		match self {
			BinaryClassificationModel::Linear(model) => {
				model::BinaryClassificationModel::Linear(model.into())
			}
			BinaryClassificationModel::Tree(model) => {
				model::BinaryClassificationModel::Tree(model.into())
			}
		}
	}
}

impl Into<model::LinearBinaryClassifier> for LinearBinaryClassificationModel {
	fn into(self) -> model::LinearBinaryClassifier {
		model::LinearBinaryClassifier {
			bias: self.model.bias,
			weights: self.model.weights.into_raw_vec(),
			means: self.model.means,
			train_options: self.train_options.into(),
			feature_groups: self.feature_groups.into_iter().map(Into::into).collect(),
			losses: self.losses,
			feature_importances: self.feature_importances,
		}
	}
}

impl Into<model::TreeBinaryClassifier> for TreeBinaryClassificationModel {
	fn into(self) -> model::TreeBinaryClassifier {
		model::TreeBinaryClassifier {
			bias: self.model.bias,
			train_options: self.train_options.into(),
			feature_groups: self.feature_groups.into_iter().map(Into::into).collect(),
			losses: self.losses,
			trees: self.model.trees.into_iter().map(Into::into).collect(),
			feature_importances: self.feature_importances,
		}
	}
}

impl Into<model::MulticlassClassificationModel> for MulticlassClassificationModel {
	fn into(self) -> model::MulticlassClassificationModel {
		match self {
			MulticlassClassificationModel::Linear(model) => {
				model::MulticlassClassificationModel::Linear(model.into())
			}
			MulticlassClassificationModel::Tree(model) => {
				model::MulticlassClassificationModel::Tree(model.into())
			}
		}
	}
}

impl Into<model::LinearMulticlassClassifier> for LinearMulticlassClassificationModel {
	fn into(self) -> model::LinearMulticlassClassifier {
		let n_features = self.model.weights.nrows();
		let n_classes = self.model.weights.ncols();
		model::LinearMulticlassClassifier {
			n_features,
			n_classes,
			biases: self.model.biases.into_raw_vec(),
			weights: self.model.weights.into_raw_vec(),
			means: self.model.means,
			train_options: self.train_options.into(),
			feature_groups: self.feature_groups.into_iter().map(Into::into).collect(),
			losses: self.losses,
			feature_importances: self.feature_importances,
		}
	}
}

impl Into<model::TreeMulticlassClassifier> for TreeMulticlassClassificationModel {
	fn into(self) -> model::TreeMulticlassClassifier {
		model::TreeMulticlassClassifier {
			n_classes: self.model.n_classes,
			n_rounds: self.model.n_rounds,
			biases: self.model.biases,
			trees: self.model.trees.into_iter().map(Into::into).collect(),
			train_options: self.train_options.into(),
			feature_groups: self.feature_groups.into_iter().map(Into::into).collect(),
			losses: self.losses,
			feature_importances: self.feature_importances,
		}
	}
}

impl Into<model::LinearModelTrainOptions> for tangram_linear::TrainOptions {
	fn into(self) -> model::LinearModelTrainOptions {
		model::LinearModelTrainOptions {
			compute_loss: self.compute_losses,
			l2_regularization: self.l2_regularization,
			learning_rate: self.learning_rate,
			max_epochs: self.max_epochs.to_u64().unwrap(),
			n_examples_per_batch: self.n_examples_per_batch.to_u64().unwrap(),
			early_stopping_options: self.early_stopping_options.map(Into::into),
		}
	}
}

impl Into<model::TreeModelTrainOptions> for tangram_tree::TrainOptions {
	fn into(self) -> model::TreeModelTrainOptions {
		model::TreeModelTrainOptions {
			binned_features_layout: self.binned_features_layout.into(),
			compute_loss: self.compute_losses,
			l2_regularization: self.l2_regularization,
			learning_rate: self.learning_rate,
			max_depth: self.max_depth.map(|max_depth| max_depth.to_u64().unwrap()),
			max_rounds: self.max_rounds.to_u64().unwrap(),
			early_stopping_options: self.early_stopping_options.map(Into::into),
			max_examples_for_computing_bin_thresholds: self
				.max_examples_for_computing_bin_thresholds
				.to_u64()
				.unwrap(),
			max_leaf_nodes: self.max_leaf_nodes.to_u64().unwrap(),
			max_valid_bins_for_number_features: self.max_valid_bins_for_number_features,
			min_examples_per_node: self.min_examples_per_node.to_u64().unwrap(),
			min_gain_to_split: self.min_gain_to_split,
			min_sum_hessians_per_node: self.min_sum_hessians_per_node,
			smoothing_factor_for_discrete_bin_sorting: self
				.smoothing_factor_for_discrete_bin_sorting,
			supplemental_l2_regularization_for_discrete_splits: self
				.supplemental_l2_regularization_for_discrete_splits,
		}
	}
}

impl Into<model::BinnedFeaturesLayout> for tangram_tree::BinnedFeaturesLayout {
	fn into(self) -> model::BinnedFeaturesLayout {
		match self {
			tangram_tree::BinnedFeaturesLayout::RowMajor => model::BinnedFeaturesLayout::RowMajor,
			tangram_tree::BinnedFeaturesLayout::ColumnMajor => {
				model::BinnedFeaturesLayout::ColumnMajor
			}
		}
	}
}

impl Into<model::LinearEarlyStoppingOptions> for tangram_linear::EarlyStoppingOptions {
	fn into(self) -> model::LinearEarlyStoppingOptions {
		model::LinearEarlyStoppingOptions {
			early_stopping_fraction: self.early_stopping_fraction,
			n_epochs_without_improvement_to_stop: self
				.n_epochs_without_improvement_to_stop
				.to_u64()
				.unwrap(),
			min_decrease_in_loss_for_significant_change: self
				.min_decrease_in_loss_for_significant_change,
		}
	}
}

impl Into<model::TreeEarlyStoppingOptions> for tangram_tree::EarlyStoppingOptions {
	fn into(self) -> model::TreeEarlyStoppingOptions {
		model::TreeEarlyStoppingOptions {
			early_stopping_fraction: self.early_stopping_fraction,
			n_epochs_without_improvement_to_stop: self
				.n_epochs_without_improvement_to_stop
				.to_u64()
				.unwrap(),
			min_decrease_in_loss_for_significant_change: self
				.min_decrease_in_loss_for_significant_change,
		}
	}
}

impl Into<model::RegressionComparisonMetric> for RegressionComparisonMetric {
	fn into(self) -> model::RegressionComparisonMetric {
		match self {
			RegressionComparisonMetric::MeanAbsoluteError => {
				model::RegressionComparisonMetric::MeanAbsoluteError
			}
			RegressionComparisonMetric::MeanSquaredError => {
				model::RegressionComparisonMetric::MeanSquaredError
			}
			RegressionComparisonMetric::RootMeanSquaredError => {
				model::RegressionComparisonMetric::RootMeanSquaredError
			}
			RegressionComparisonMetric::R2 => model::RegressionComparisonMetric::R2,
		}
	}
}

impl Into<model::MulticlassClassificationComparisonMetric>
	for MulticlassClassificationComparisonMetric
{
	fn into(self) -> model::MulticlassClassificationComparisonMetric {
		match self {
			MulticlassClassificationComparisonMetric::Accuracy => {
				model::MulticlassClassificationComparisonMetric::Accuracy
			}
		}
	}
}

impl Into<model::BinaryClassificationComparisonMetric> for BinaryClassificationComparisonMetric {
	fn into(self) -> model::BinaryClassificationComparisonMetric {
		match self {
			BinaryClassificationComparisonMetric::AUCROC => {
				model::BinaryClassificationComparisonMetric::AUCROC
			}
		}
	}
}
