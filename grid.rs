use crate::{config, features, stats};
use itertools::iproduct;

pub enum GridItem {
	LinearRegressor {
		target_column_index: usize,
		feature_groups: Vec<features::FeatureGroup>,
		options: LinearModelTrainOptions,
	},
	GBTRegressor {
		target_column_index: usize,
		feature_groups: Vec<features::FeatureGroup>,
		options: GBTModelTrainOptions,
	},
	LinearBinaryClassifier {
		target_column_index: usize,
		feature_groups: Vec<features::FeatureGroup>,
		options: LinearModelTrainOptions,
	},
	GBTBinaryClassifier {
		target_column_index: usize,
		feature_groups: Vec<features::FeatureGroup>,
		options: GBTModelTrainOptions,
	},
	LinearMulticlassClassifier {
		target_column_index: usize,
		feature_groups: Vec<features::FeatureGroup>,
		options: LinearModelTrainOptions,
	},
	GBTMulticlassClassifier {
		target_column_index: usize,
		feature_groups: Vec<features::FeatureGroup>,
		options: GBTModelTrainOptions,
	},
}

pub struct LinearModelTrainOptions {
	pub max_epochs: u64,
	pub n_examples_per_batch: u64,
	pub learning_rate: f32,
	pub l2_regularization: f32,
	pub early_stopping_fraction: f32,
}

pub struct GBTModelTrainOptions {
	pub max_depth: u64,
	pub learning_rate: f32,
	pub l2_regularization: f32,
	pub min_examples_per_leaf: u64,
	pub max_rounds: u64,
	pub early_stopping_fraction: f32,
}

pub fn compute_regression_hyperparameter_grid(
	grid: &[config::GridItem],
	target_column_index: usize,
	column_stats: &[stats::ColumnStats],
) -> Vec<GridItem> {
	grid.iter()
		.map(|item| match item {
			config::GridItem::Linear(item) => GridItem::LinearRegressor {
				target_column_index,
				feature_groups: features::compute_feature_groups_linear(column_stats),
				options: LinearModelTrainOptions {
					early_stopping_fraction: 0.1,
					l2_regularization: item.l2_regularization,
					learning_rate: item.learning_rate,
					max_epochs: item.max_epochs,
					n_examples_per_batch: item.n_examples_per_batch,
				},
			},
			config::GridItem::GBT(item) => GridItem::GBTRegressor {
				target_column_index,
				feature_groups: features::compute_feature_groups_gbt(column_stats),
				options: GBTModelTrainOptions {
					max_depth: item.max_depth,
					early_stopping_fraction: 0.1,
					learning_rate: item.learning_rate,
					l2_regularization: item.l2_regularization,
					min_examples_per_leaf: item.min_examples_per_leaf,
					max_rounds: item.max_rounds,
				},
			},
		})
		.collect()
}

pub fn compute_binary_classification_hyperparameter_grid(
	grid: &[config::GridItem],
	target_column_index: usize,
	column_stats: &[stats::ColumnStats],
) -> Vec<GridItem> {
	grid.iter()
		.map(|item| match item {
			config::GridItem::Linear(item) => GridItem::LinearBinaryClassifier {
				target_column_index,
				feature_groups: features::compute_feature_groups_linear(column_stats),
				options: LinearModelTrainOptions {
					early_stopping_fraction: 0.1,
					l2_regularization: item.l2_regularization,
					learning_rate: item.learning_rate,
					max_epochs: item.max_epochs,
					n_examples_per_batch: item.n_examples_per_batch,
				},
			},
			config::GridItem::GBT(item) => GridItem::GBTBinaryClassifier {
				target_column_index,
				feature_groups: features::compute_feature_groups_gbt(column_stats),
				options: GBTModelTrainOptions {
					max_depth: item.max_depth,
					early_stopping_fraction: 0.1,
					learning_rate: item.learning_rate,
					min_examples_per_leaf: item.min_examples_per_leaf,
					max_rounds: item.max_rounds,
					l2_regularization: item.l2_regularization,
				},
			},
		})
		.collect()
}

pub fn compute_multiclass_classification_hyperparameter_grid(
	grid: &[config::GridItem],
	target_column_index: usize,
	column_stats: &[stats::ColumnStats],
) -> Vec<GridItem> {
	grid.iter()
		.map(|item| match item {
			config::GridItem::Linear(item) => GridItem::LinearMulticlassClassifier {
				target_column_index,
				feature_groups: features::compute_feature_groups_linear(column_stats),
				options: LinearModelTrainOptions {
					early_stopping_fraction: 0.1,
					l2_regularization: item.l2_regularization,
					learning_rate: item.learning_rate,
					max_epochs: item.max_epochs,
					n_examples_per_batch: item.n_examples_per_batch,
				},
			},
			config::GridItem::GBT(item) => GridItem::GBTMulticlassClassifier {
				target_column_index,
				feature_groups: features::compute_feature_groups_gbt(column_stats),
				options: GBTModelTrainOptions {
					max_depth: item.max_depth,
					early_stopping_fraction: 0.1,
					learning_rate: item.learning_rate,
					min_examples_per_leaf: item.min_examples_per_leaf,
					max_rounds: item.max_rounds,
					l2_regularization: item.l2_regularization,
				},
			},
		})
		.collect()
}

const DEFAULT_LINEAR_MODEL_LEARNING_RATE_VALUES: [f32; 4] = [0.1, 0.01, 0.001, 0.0001];
const DEFAULT_LINEAR_L2_REGULARIZATION_VALUES: [f32; 6] = [1.0, 0.1, 0.01, 0.001, 0.0001, 0.0];
const DEFAULT_LINEAR_MAX_EPOCHS_VALUES: [u64; 1] = [100];
const DEFAULT_LINEAR_N_EXAMPLES_PER_BATCH_VALUES: [u64; 1] = [128];

const DEFAULT_GBT_LEARNING_RATE_VALUES: [f32; 3] = [0.1, 0.01, 0.001];
const DEFAULT_GBT_L2_REGULARIZATION_VALUES: [f32; 6] = [1.0, 0.1, 0.01, 0.001, 0.0001, 0.0];
const DEFAULT_GBT_DEPTH_VALUES: [u64; 2] = [3, 6];
const DEFAULT_GBT_MAX_TREES_VALUES: [u64; 2] = [100, 1000];
const DEFAULT_GBT_MIN_EXAMPLES_PER_LEAF_VALUES: [u64; 1] = [10];

pub fn default_regression_hyperparameter_grid(
	target_column_index: usize,
	column_stats: &[stats::ColumnStats],
) -> Vec<GridItem> {
	let mut grid = Vec::new();
	for (&l2_regularization, &learning_rate, &max_epochs, &n_examples_per_batch) in iproduct!(
		DEFAULT_LINEAR_L2_REGULARIZATION_VALUES.iter(),
		DEFAULT_LINEAR_MODEL_LEARNING_RATE_VALUES.iter(),
		DEFAULT_LINEAR_MAX_EPOCHS_VALUES.iter(),
		DEFAULT_LINEAR_N_EXAMPLES_PER_BATCH_VALUES.iter()
	) {
		grid.push(GridItem::LinearRegressor {
			target_column_index,
			feature_groups: features::compute_feature_groups_linear(column_stats),
			options: LinearModelTrainOptions {
				l2_regularization,
				learning_rate,
				max_epochs,
				n_examples_per_batch,
				early_stopping_fraction: 0.1,
			},
		});
	}
	for (&max_depth, &learning_rate, &l2_regularization, &min_examples_per_leaf, &max_rounds) in iproduct!(
		DEFAULT_GBT_DEPTH_VALUES.iter(),
		DEFAULT_GBT_LEARNING_RATE_VALUES.iter(),
		DEFAULT_GBT_L2_REGULARIZATION_VALUES.iter(),
		DEFAULT_GBT_MIN_EXAMPLES_PER_LEAF_VALUES.iter(),
		DEFAULT_GBT_MAX_TREES_VALUES.iter()
	) {
		grid.push(GridItem::GBTRegressor {
			target_column_index,
			feature_groups: features::compute_feature_groups_gbt(column_stats),
			options: GBTModelTrainOptions {
				max_depth,
				learning_rate,
				min_examples_per_leaf,
				max_rounds,
				early_stopping_fraction: 0.1,
				l2_regularization,
			},
		});
	}
	grid
}

pub fn default_binary_classification_hyperparameter_grid(
	target_column_index: usize,
	column_stats: &[stats::ColumnStats],
) -> Vec<GridItem> {
	let mut grid = Vec::new();
	for (&l2_regularization, &learning_rate, &max_epochs, &n_examples_per_batch) in iproduct!(
		DEFAULT_LINEAR_L2_REGULARIZATION_VALUES.iter(),
		DEFAULT_LINEAR_MODEL_LEARNING_RATE_VALUES.iter(),
		DEFAULT_LINEAR_MAX_EPOCHS_VALUES.iter(),
		DEFAULT_LINEAR_N_EXAMPLES_PER_BATCH_VALUES.iter()
	) {
		grid.push(GridItem::LinearBinaryClassifier {
			target_column_index,
			feature_groups: features::compute_feature_groups_linear(column_stats),
			options: LinearModelTrainOptions {
				l2_regularization,
				learning_rate,
				max_epochs,
				n_examples_per_batch,
				early_stopping_fraction: 0.1,
			},
		});
	}
	for (&max_depth, &learning_rate, &l2_regularization, &min_examples_per_leaf, &max_rounds) in iproduct!(
		DEFAULT_GBT_DEPTH_VALUES.iter(),
		DEFAULT_GBT_LEARNING_RATE_VALUES.iter(),
		DEFAULT_GBT_L2_REGULARIZATION_VALUES.iter(),
		DEFAULT_GBT_MIN_EXAMPLES_PER_LEAF_VALUES.iter(),
		DEFAULT_GBT_MAX_TREES_VALUES.iter()
	) {
		grid.push(GridItem::GBTBinaryClassifier {
			target_column_index,
			feature_groups: features::compute_feature_groups_gbt(column_stats),
			options: GBTModelTrainOptions {
				max_depth,
				learning_rate,
				min_examples_per_leaf,
				max_rounds,
				early_stopping_fraction: 0.1,
				l2_regularization,
			},
		});
	}
	grid
}

pub fn default_multiclass_classification_hyperparameter_grid(
	target_column_index: usize,
	column_stats: &[stats::ColumnStats],
) -> Vec<GridItem> {
	let mut grid = Vec::new();
	for (&l2_regularization, &learning_rate, &max_epochs, &n_examples_per_batch) in iproduct!(
		DEFAULT_LINEAR_L2_REGULARIZATION_VALUES.iter(),
		DEFAULT_LINEAR_MODEL_LEARNING_RATE_VALUES.iter(),
		DEFAULT_LINEAR_MAX_EPOCHS_VALUES.iter(),
		DEFAULT_LINEAR_N_EXAMPLES_PER_BATCH_VALUES.iter()
	) {
		grid.push(GridItem::LinearMulticlassClassifier {
			target_column_index,
			feature_groups: features::compute_feature_groups_linear(column_stats),
			options: LinearModelTrainOptions {
				l2_regularization,
				learning_rate,
				max_epochs,
				n_examples_per_batch,
				early_stopping_fraction: 0.1,
			},
		});
	}
	for (&max_depth, &learning_rate, &l2_regularization, &min_examples_per_leaf, &max_rounds) in iproduct!(
		DEFAULT_GBT_DEPTH_VALUES.iter(),
		DEFAULT_GBT_LEARNING_RATE_VALUES.iter(),
		DEFAULT_GBT_L2_REGULARIZATION_VALUES.iter(),
		DEFAULT_GBT_MIN_EXAMPLES_PER_LEAF_VALUES.iter(),
		DEFAULT_GBT_MAX_TREES_VALUES.iter()
	) {
		grid.push(GridItem::GBTMulticlassClassifier {
			target_column_index,
			feature_groups: features::compute_feature_groups_gbt(column_stats),
			options: GBTModelTrainOptions {
				max_depth,
				learning_rate,
				min_examples_per_leaf,
				max_rounds,
				early_stopping_fraction: 0.1,
				l2_regularization,
			},
		});
	}
	grid
}
