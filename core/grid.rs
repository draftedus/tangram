use crate::{config, features, stats};
use itertools::iproduct;

/// A `GridItem` is a description of a single entry in a hyperparameter grid. It specifies what feature engineering to perform on the training data, which model to train, and which hyperparameters to use.
pub enum GridItem {
	LinearRegressor {
		target_column_index: usize,
		feature_groups: Vec<features::FeatureGroup>,
		options: LinearModelTrainOptions,
	},
	TreeRegressor {
		target_column_index: usize,
		feature_groups: Vec<features::FeatureGroup>,
		options: TreeModelTrainOptions,
	},
	LinearBinaryClassifier {
		target_column_index: usize,
		feature_groups: Vec<features::FeatureGroup>,
		options: LinearModelTrainOptions,
	},
	TreeBinaryClassifier {
		target_column_index: usize,
		feature_groups: Vec<features::FeatureGroup>,
		options: TreeModelTrainOptions,
	},
	LinearMulticlassClassifier {
		target_column_index: usize,
		feature_groups: Vec<features::FeatureGroup>,
		options: LinearModelTrainOptions,
	},
	TreeMulticlassClassifier {
		target_column_index: usize,
		feature_groups: Vec<features::FeatureGroup>,
		options: TreeModelTrainOptions,
	},
}

pub struct LinearModelTrainOptions {
	pub l2_regularization: Option<f32>,
	pub learning_rate: Option<f32>,
	pub max_epochs: Option<u64>,
	pub n_examples_per_batch: Option<u64>,
}

pub struct TreeModelTrainOptions {
	pub l2_regularization: Option<f32>,
	pub learning_rate: Option<f32>,
	pub max_depth: Option<u64>,
	pub max_rounds: Option<u64>,
	pub min_examples_per_leaf: Option<u64>,
}

pub fn compute_regression_hyperparameter_grid(
	grid: &[config::GridItem],
	target_column_index: usize,
	column_stats: &[stats::ColumnStatsOutput],
) -> Vec<GridItem> {
	grid.iter()
		.map(|item| match item {
			config::GridItem::Linear(item) => GridItem::LinearRegressor {
				target_column_index,
				feature_groups: features::compute_feature_groups_linear(column_stats),
				options: LinearModelTrainOptions {
					l2_regularization: item.l2_regularization,
					learning_rate: item.learning_rate,
					max_epochs: item.max_epochs,
					n_examples_per_batch: item.n_examples_per_batch,
				},
			},
			config::GridItem::Tree(item) => GridItem::TreeRegressor {
				target_column_index,
				feature_groups: features::compute_feature_groups_tree(column_stats),
				options: TreeModelTrainOptions {
					max_depth: item.max_depth,
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
	column_stats: &[stats::ColumnStatsOutput],
) -> Vec<GridItem> {
	grid.iter()
		.map(|item| match item {
			config::GridItem::Linear(item) => GridItem::LinearBinaryClassifier {
				target_column_index,
				feature_groups: features::compute_feature_groups_linear(column_stats),
				options: LinearModelTrainOptions {
					l2_regularization: item.l2_regularization,
					learning_rate: item.learning_rate,
					max_epochs: item.max_epochs,
					n_examples_per_batch: item.n_examples_per_batch,
				},
			},
			config::GridItem::Tree(item) => GridItem::TreeBinaryClassifier {
				target_column_index,
				feature_groups: features::compute_feature_groups_tree(column_stats),
				options: TreeModelTrainOptions {
					max_depth: item.max_depth,
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
	column_stats: &[stats::ColumnStatsOutput],
) -> Vec<GridItem> {
	grid.iter()
		.map(|item| match item {
			config::GridItem::Linear(item) => GridItem::LinearMulticlassClassifier {
				target_column_index,
				feature_groups: features::compute_feature_groups_linear(column_stats),
				options: LinearModelTrainOptions {
					l2_regularization: item.l2_regularization,
					learning_rate: item.learning_rate,
					max_epochs: item.max_epochs,
					n_examples_per_batch: item.n_examples_per_batch,
				},
			},
			config::GridItem::Tree(item) => GridItem::TreeMulticlassClassifier {
				target_column_index,
				feature_groups: features::compute_feature_groups_tree(column_stats),
				options: TreeModelTrainOptions {
					max_depth: item.max_depth,
					learning_rate: item.learning_rate,
					min_examples_per_leaf: item.min_examples_per_leaf,
					max_rounds: item.max_rounds,
					l2_regularization: item.l2_regularization,
				},
			},
		})
		.collect()
}

// TODO

// const DEFAULT_LINEAR_MODEL_LEARNING_RATE_VALUES: [f32; 4] = [0.1, 0.01, 0.001, 0.0001];
// const DEFAULT_LINEAR_L2_REGULARIZATION_VALUES: [f32; 6] = [1.0, 0.1, 0.01, 0.001, 0.0001, 0.0];
// const DEFAULT_LINEAR_MAX_EPOCHS_VALUES: [u64; 1] = [100];
// const DEFAULT_LINEAR_N_EXAMPLES_PER_BATCH_VALUES: [u64; 1] = [128];
const DEFAULT_LINEAR_MODEL_LEARNING_RATE_VALUES: [f32; 1] = [0.1];
const DEFAULT_LINEAR_L2_REGULARIZATION_VALUES: [f32; 1] = [1.0];
const DEFAULT_LINEAR_MAX_EPOCHS_VALUES: [u64; 1] = [100];
const DEFAULT_LINEAR_N_EXAMPLES_PER_BATCH_VALUES: [u64; 1] = [128];

// const DEFAULT_TREE_LEARNING_RATE_VALUES: [f32; 3] = [0.1, 0.01, 0.001];
// const DEFAULT_TREE_L2_REGULARIZATION_VALUES: [f32; 6] = [1.0, 0.1, 0.01, 0.001, 0.0001, 0.0];
// const DEFAULT_TREE_DEPTH_VALUES: [u64; 2] = [3, 6];
// const DEFAULT_TREE_MAX_TREES_VALUES: [u64; 2] = [100, 1000];
// const DEFAULT_TREE_MIN_EXAMPLES_PER_LEAF_VALUES: [u64; 1] = [10];
const DEFAULT_TREE_LEARNING_RATE_VALUES: [f32; 1] = [0.1];
const DEFAULT_TREE_L2_REGULARIZATION_VALUES: [f32; 1] = [1.0];
const DEFAULT_TREE_DEPTH_VALUES: [u64; 1] = [3];
const DEFAULT_TREE_MAX_TREES_VALUES: [u64; 1] = [100];
const DEFAULT_TREE_MIN_EXAMPLES_PER_LEAF_VALUES: [u64; 1] = [10];

/// Compute the default hyperparameter grid for regression.
pub fn default_regression_hyperparameter_grid(
	target_column_index: usize,
	column_stats: &[stats::ColumnStatsOutput],
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
				l2_regularization: Some(l2_regularization),
				learning_rate: Some(learning_rate),
				max_epochs: Some(max_epochs),
				n_examples_per_batch: Some(n_examples_per_batch),
			},
		});
	}
	for (&max_depth, &learning_rate, &l2_regularization, &min_examples_per_leaf, &max_rounds) in iproduct!(
		DEFAULT_TREE_DEPTH_VALUES.iter(),
		DEFAULT_TREE_LEARNING_RATE_VALUES.iter(),
		DEFAULT_TREE_L2_REGULARIZATION_VALUES.iter(),
		DEFAULT_TREE_MIN_EXAMPLES_PER_LEAF_VALUES.iter(),
		DEFAULT_TREE_MAX_TREES_VALUES.iter()
	) {
		grid.push(GridItem::TreeRegressor {
			target_column_index,
			feature_groups: features::compute_feature_groups_tree(column_stats),
			options: TreeModelTrainOptions {
				max_depth: Some(max_depth),
				learning_rate: Some(learning_rate),
				min_examples_per_leaf: Some(min_examples_per_leaf),
				max_rounds: Some(max_rounds),
				l2_regularization: Some(l2_regularization),
			},
		});
	}
	grid
}

/// Compute the default hyperparameter grid for binary classification.
pub fn default_binary_classification_hyperparameter_grid(
	target_column_index: usize,
	column_stats: &[stats::ColumnStatsOutput],
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
				l2_regularization: Some(l2_regularization),
				learning_rate: Some(learning_rate),
				max_epochs: Some(max_epochs),
				n_examples_per_batch: Some(n_examples_per_batch),
			},
		});
	}
	for (&max_depth, &learning_rate, &l2_regularization, &min_examples_per_leaf, &max_rounds) in iproduct!(
		DEFAULT_TREE_DEPTH_VALUES.iter(),
		DEFAULT_TREE_LEARNING_RATE_VALUES.iter(),
		DEFAULT_TREE_L2_REGULARIZATION_VALUES.iter(),
		DEFAULT_TREE_MIN_EXAMPLES_PER_LEAF_VALUES.iter(),
		DEFAULT_TREE_MAX_TREES_VALUES.iter()
	) {
		grid.push(GridItem::TreeBinaryClassifier {
			target_column_index,
			feature_groups: features::compute_feature_groups_tree(column_stats),
			options: TreeModelTrainOptions {
				max_depth: Some(max_depth),
				learning_rate: Some(learning_rate),
				min_examples_per_leaf: Some(min_examples_per_leaf),
				max_rounds: Some(max_rounds),
				l2_regularization: Some(l2_regularization),
			},
		});
	}
	grid
}

/// Compute the default hyperparameter grid for multiclass classification.
pub fn default_multiclass_classification_hyperparameter_grid(
	target_column_index: usize,
	column_stats: &[stats::ColumnStatsOutput],
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
				l2_regularization: Some(l2_regularization),
				learning_rate: Some(learning_rate),
				max_epochs: Some(max_epochs),
				n_examples_per_batch: Some(n_examples_per_batch),
			},
		});
	}
	for (&max_depth, &learning_rate, &l2_regularization, &min_examples_per_leaf, &max_rounds) in iproduct!(
		DEFAULT_TREE_DEPTH_VALUES.iter(),
		DEFAULT_TREE_LEARNING_RATE_VALUES.iter(),
		DEFAULT_TREE_L2_REGULARIZATION_VALUES.iter(),
		DEFAULT_TREE_MIN_EXAMPLES_PER_LEAF_VALUES.iter(),
		DEFAULT_TREE_MAX_TREES_VALUES.iter()
	) {
		grid.push(GridItem::TreeMulticlassClassifier {
			target_column_index,
			feature_groups: features::compute_feature_groups_tree(column_stats),
			options: TreeModelTrainOptions {
				max_depth: Some(max_depth),
				learning_rate: Some(learning_rate),
				min_examples_per_leaf: Some(min_examples_per_leaf),
				max_rounds: Some(max_rounds),
				l2_regularization: Some(l2_regularization),
			},
		});
	}
	grid
}
