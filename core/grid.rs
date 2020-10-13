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
	pub early_stopping_options: Option<EarlyStoppingOptions>,
}

impl Default for LinearModelTrainOptions {
	fn default() -> Self {
		Self {
			l2_regularization: None,
			learning_rate: None,
			max_epochs: None,
			n_examples_per_batch: None,
			early_stopping_options: None,
		}
	}
}

pub struct TreeModelTrainOptions {
	pub early_stopping_options: Option<EarlyStoppingOptions>,
	pub l2_regularization: Option<f32>,
	pub learning_rate: Option<f32>,
	pub max_depth: Option<u64>,
	pub max_examples_for_computing_bin_thresholds: Option<usize>,
	pub max_leaf_nodes: Option<usize>,
	pub max_rounds: Option<u64>,
	pub max_valid_bins_for_number_features: Option<u8>,
	pub min_examples_per_node: Option<u64>,
	pub min_gain_to_split: Option<f32>,
	pub min_sum_hessians_per_node: Option<f32>,
	pub smoothing_factor_for_discrete_bin_sorting: Option<f32>,
	pub supplemental_l2_regularization_for_discrete_splits: Option<f32>,
}

impl Default for TreeModelTrainOptions {
	fn default() -> Self {
		Self {
			early_stopping_options: None,
			l2_regularization: None,
			learning_rate: None,
			max_depth: None,
			max_examples_for_computing_bin_thresholds: None,
			max_leaf_nodes: None,
			max_rounds: None,
			max_valid_bins_for_number_features: None,
			min_examples_per_node: None,
			min_gain_to_split: None,
			min_sum_hessians_per_node: None,
			smoothing_factor_for_discrete_bin_sorting: None,
			supplemental_l2_regularization_for_discrete_splits: None,
		}
	}
}

pub struct EarlyStoppingOptions {
	pub early_stopping_fraction: f32,
	pub early_stopping_rounds: usize,
	pub early_stopping_threshold: f32,
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
					early_stopping_options: item.early_stopping_options.as_ref().map(
						|early_stopping_options| EarlyStoppingOptions {
							early_stopping_fraction: early_stopping_options.early_stopping_fraction,
							early_stopping_rounds: early_stopping_options.early_stopping_rounds,
							early_stopping_threshold: early_stopping_options
								.early_stopping_threshold,
						},
					),
				},
			},
			config::GridItem::Tree(item) => GridItem::TreeRegressor {
				target_column_index,
				feature_groups: features::compute_feature_groups_tree(column_stats),
				options: TreeModelTrainOptions {
					max_depth: item.max_depth,
					learning_rate: item.learning_rate,
					l2_regularization: item.l2_regularization,
					min_examples_per_node: item.min_examples_per_node,
					max_rounds: item.max_rounds,
					early_stopping_options: item.early_stopping_options.as_ref().map(
						|early_stopping_options| EarlyStoppingOptions {
							early_stopping_fraction: early_stopping_options.early_stopping_fraction,
							early_stopping_rounds: early_stopping_options.early_stopping_rounds,
							early_stopping_threshold: early_stopping_options
								.early_stopping_threshold,
						},
					),
					max_examples_for_computing_bin_thresholds: item
						.max_examples_for_computing_bin_thresholds,
					max_leaf_nodes: item.max_leaf_nodes,
					max_valid_bins_for_number_features: item.max_valid_bins_for_number_features,
					min_gain_to_split: item.min_gain_to_split,
					min_sum_hessians_per_node: item.min_sum_hessians_per_node,
					smoothing_factor_for_discrete_bin_sorting: item
						.smoothing_factor_for_discrete_bin_sorting,
					supplemental_l2_regularization_for_discrete_splits: item
						.supplemental_l2_regularization_for_discrete_splits,
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
					early_stopping_options: item.early_stopping_options.as_ref().map(
						|early_stopping_options| EarlyStoppingOptions {
							early_stopping_fraction: early_stopping_options.early_stopping_fraction,
							early_stopping_rounds: early_stopping_options.early_stopping_rounds,
							early_stopping_threshold: early_stopping_options
								.early_stopping_threshold,
						},
					),
				},
			},
			config::GridItem::Tree(item) => GridItem::TreeBinaryClassifier {
				target_column_index,
				feature_groups: features::compute_feature_groups_tree(column_stats),
				options: TreeModelTrainOptions {
					max_depth: item.max_depth,
					learning_rate: item.learning_rate,
					min_examples_per_node: item.min_examples_per_node,
					max_rounds: item.max_rounds,
					l2_regularization: item.l2_regularization,
					early_stopping_options: item.early_stopping_options.as_ref().map(
						|early_stopping_options| EarlyStoppingOptions {
							early_stopping_fraction: early_stopping_options.early_stopping_fraction,
							early_stopping_rounds: early_stopping_options.early_stopping_rounds,
							early_stopping_threshold: early_stopping_options
								.early_stopping_threshold,
						},
					),
					max_examples_for_computing_bin_thresholds: item
						.max_examples_for_computing_bin_thresholds,
					max_leaf_nodes: item.max_leaf_nodes,
					max_valid_bins_for_number_features: item.max_valid_bins_for_number_features,
					min_gain_to_split: item.min_gain_to_split,
					min_sum_hessians_per_node: item.min_sum_hessians_per_node,
					smoothing_factor_for_discrete_bin_sorting: item
						.smoothing_factor_for_discrete_bin_sorting,
					supplemental_l2_regularization_for_discrete_splits: item
						.supplemental_l2_regularization_for_discrete_splits,
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
					early_stopping_options: item.early_stopping_options.as_ref().map(
						|early_stopping_options| EarlyStoppingOptions {
							early_stopping_fraction: early_stopping_options.early_stopping_fraction,
							early_stopping_rounds: early_stopping_options.early_stopping_rounds,
							early_stopping_threshold: early_stopping_options
								.early_stopping_threshold,
						},
					),
				},
			},
			config::GridItem::Tree(item) => GridItem::TreeMulticlassClassifier {
				target_column_index,
				feature_groups: features::compute_feature_groups_tree(column_stats),
				options: TreeModelTrainOptions {
					max_depth: item.max_depth,
					learning_rate: item.learning_rate,
					min_examples_per_node: item.min_examples_per_node,
					max_rounds: item.max_rounds,
					l2_regularization: item.l2_regularization,
					early_stopping_options: item.early_stopping_options.as_ref().map(
						|early_stopping_options| EarlyStoppingOptions {
							early_stopping_fraction: early_stopping_options.early_stopping_fraction,
							early_stopping_rounds: early_stopping_options.early_stopping_rounds,
							early_stopping_threshold: early_stopping_options
								.early_stopping_threshold,
						},
					),
					max_examples_for_computing_bin_thresholds: item
						.max_examples_for_computing_bin_thresholds,
					max_leaf_nodes: item.max_leaf_nodes,
					max_valid_bins_for_number_features: item.max_valid_bins_for_number_features,
					min_gain_to_split: item.min_gain_to_split,
					min_sum_hessians_per_node: item.min_sum_hessians_per_node,
					smoothing_factor_for_discrete_bin_sorting: item
						.smoothing_factor_for_discrete_bin_sorting,
					supplemental_l2_regularization_for_discrete_splits: item
						.supplemental_l2_regularization_for_discrete_splits,
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
const DEFAULT_TREE_MIN_EXAMPLES_PER_NODE_VALUES: [u64; 1] = [10];

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
				..Default::default()
			},
		});
	}
	for (&max_depth, &learning_rate, &l2_regularization, &min_examples_per_node, &max_rounds) in iproduct!(
		DEFAULT_TREE_DEPTH_VALUES.iter(),
		DEFAULT_TREE_LEARNING_RATE_VALUES.iter(),
		DEFAULT_TREE_L2_REGULARIZATION_VALUES.iter(),
		DEFAULT_TREE_MIN_EXAMPLES_PER_NODE_VALUES.iter(),
		DEFAULT_TREE_MAX_TREES_VALUES.iter()
	) {
		grid.push(GridItem::TreeRegressor {
			target_column_index,
			feature_groups: features::compute_feature_groups_tree(column_stats),
			options: TreeModelTrainOptions {
				max_depth: Some(max_depth),
				learning_rate: Some(learning_rate),
				min_examples_per_node: Some(min_examples_per_node),
				max_rounds: Some(max_rounds),
				l2_regularization: Some(l2_regularization),
				..Default::default()
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
				..Default::default()
			},
		});
	}
	for (&max_depth, &learning_rate, &l2_regularization, &min_examples_per_node, &max_rounds) in iproduct!(
		DEFAULT_TREE_DEPTH_VALUES.iter(),
		DEFAULT_TREE_LEARNING_RATE_VALUES.iter(),
		DEFAULT_TREE_L2_REGULARIZATION_VALUES.iter(),
		DEFAULT_TREE_MIN_EXAMPLES_PER_NODE_VALUES.iter(),
		DEFAULT_TREE_MAX_TREES_VALUES.iter()
	) {
		grid.push(GridItem::TreeBinaryClassifier {
			target_column_index,
			feature_groups: features::compute_feature_groups_tree(column_stats),
			options: TreeModelTrainOptions {
				max_depth: Some(max_depth),
				learning_rate: Some(learning_rate),
				min_examples_per_node: Some(min_examples_per_node),
				max_rounds: Some(max_rounds),
				l2_regularization: Some(l2_regularization),
				..Default::default()
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
				..Default::default()
			},
		});
	}
	for (&max_depth, &learning_rate, &l2_regularization, &min_examples_per_node, &max_rounds) in iproduct!(
		DEFAULT_TREE_DEPTH_VALUES.iter(),
		DEFAULT_TREE_LEARNING_RATE_VALUES.iter(),
		DEFAULT_TREE_L2_REGULARIZATION_VALUES.iter(),
		DEFAULT_TREE_MIN_EXAMPLES_PER_NODE_VALUES.iter(),
		DEFAULT_TREE_MAX_TREES_VALUES.iter()
	) {
		grid.push(GridItem::TreeMulticlassClassifier {
			target_column_index,
			feature_groups: features::compute_feature_groups_tree(column_stats),
			options: TreeModelTrainOptions {
				max_depth: Some(max_depth),
				learning_rate: Some(learning_rate),
				min_examples_per_node: Some(min_examples_per_node),
				max_rounds: Some(max_rounds),
				l2_regularization: Some(l2_regularization),
				..Default::default()
			},
		});
	}
	grid
}
