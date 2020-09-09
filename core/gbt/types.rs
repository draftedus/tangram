/*!
This module contains definitions of the crate's public types.
*/

use ndarray::prelude::*;

/// The options passed to tangram_gbt::train
#[derive(Debug)]
pub struct TrainOptions {
	/// If true, the model will include the loss on the training data at each round.
	pub compute_loss: bool,
	/// l2 regularization value to use for discrete splits
	pub discrete_l2_regularization: f32,
	/// Hello world.
	pub discrete_min_examples_per_branch: usize,
	/// Hello world.
	pub discrete_smoothing_factor: f32,
	/// Specify options for early stopping. If the value is `Some`, early stopping will be enabled. If it is `None`, early stopping will be disabled.
	pub early_stopping_options: Option<EarlyStoppingOptions>,
	/// serves to help avoid overfitting. Refer to XGBOOST paper section 2.1 Regularized Learning Objective.
	pub l2_regularization: f32,
	/// We multiply the output of each tree by this value. It serves to prevent overfitting. It is known as eta in xgboost. Common values are [0.1, 0.01, 0.001]. The smaller the learning rate, the more rounds you need. > 0
	pub learning_rate: f32,
	/// maximum depth we will grow a tree. Related to max_leaf_nodes. A fully dense tree will have a maximum of 2^depth leaf nodes.
	pub max_depth: usize,
	/// maximum number of leaf nodes before stopping to train an individual tree.
	pub max_leaf_nodes: usize,
	/// maximum number of bins to use when mapping our feature values into binned features. The default value is 255 because we want to store binned indexes as u8. Follows the convention in sklearn, lightgbm and xgboost. The final bin (256) is reserved for missing values.
	pub max_non_missing_bins: u8,
	/// maximum number of rounds of boosting, could be less if we are using early stopping.  The number of trees is related to the the max_rounds. In regression and binary classification, the maximum number of trees is equal to max rounds. In multiclass classification, the maximum number of trees is num_classes * max_rounds. >= 0. If max_rounds = 0, the baseline classifier is returned, (just the bias).
	pub max_rounds: usize,
	/// the minimum number of examples that must be present in a leaf node during training in order to add the node to the tree.
	pub min_examples_leaf: usize,
	/// min_gain_to_split
	pub min_gain_to_split: f32,
	/// The minimum value of the sum of hessians to still be considered to split.
	pub min_sum_hessians_in_leaf: f32,
	/// The maximum number of examples to consider for determining the bin thresholds for number columns.
	pub subsample_for_binning: usize,
}

#[derive(Debug)]
pub struct EarlyStoppingOptions {
	// the fraction of the dataset that we should set aside for use in early stopping
	pub early_stopping_fraction: f32,
	/// the maximum number of rounds of boosting that we will do if we don't see an improvement by at least `early_stopping_threshold` in the loss.
	pub early_stopping_rounds: usize,
	/// the minimum amount a subsequent round of boosting must decrease the loss by. Early stopping can be thought of as a simple state machine: If we have a round that doesn't decrease the loss by at least tol, we increment our counter. If we decrease the loss by at least tol, the counter is reset to 0. If the counter hits early_stopping_rounds rounds, we stop training the tree.
	pub early_stopping_threshold: f32,
}

impl Default for TrainOptions {
	fn default() -> Self {
		Self {
			compute_loss: false,
			early_stopping_options: None,
			l2_regularization: 0.0,
			learning_rate: 0.1,
			max_depth: 5,
			max_leaf_nodes: 31,
			max_non_missing_bins: 255,
			subsample_for_binning: 200_000,
			max_rounds: 100,
			min_examples_leaf: 20,
			min_sum_hessians_in_leaf: 1e-3,
			min_gain_to_split: 0.0,
			discrete_smoothing_factor: 10.0,
			discrete_l2_regularization: 10.0,
			discrete_min_examples_per_branch: 100,
		}
	}
}

#[derive(Debug)]
pub enum Task {
	Regression,
	BinaryClassification,
	MulticlassClassification { n_trees_per_round: usize },
}

#[derive(Debug)]
pub enum Model {
	Regressor(Regressor),
	BinaryClassifier(BinaryClassifier),
	MulticlassClassifier(MulticlassClassifier),
}

#[derive(Debug)]
pub struct Regressor {
	pub bias: f32,
	pub trees: Vec<Tree>,
	pub feature_importances: Option<Array1<f32>>,
	pub losses: Option<Array1<f32>>,
}

#[derive(Debug)]
pub struct BinaryClassifier {
	pub bias: f32,
	pub trees: Vec<Tree>,
	pub feature_importances: Option<Array1<f32>>,
	pub losses: Option<Array1<f32>>,
	pub classes: Vec<String>,
}

#[derive(Debug)]
pub struct MulticlassClassifier {
	pub biases: Vec<f32>,
	/// (n_rounds, n_classes)
	pub trees: Vec<Tree>,
	pub n_classes: usize,
	pub n_rounds: usize,
	pub feature_importances: Option<Array1<f32>>,
	pub losses: Option<Array1<f32>>,
	pub classes: Vec<String>,
}

#[derive(Debug)]
pub struct Tree {
	pub nodes: Vec<Node>,
}

#[derive(Debug)]
pub enum Node {
	Branch(BranchNode),
	Leaf(LeafNode),
}

impl Node {
	pub fn examples_fraction(&self) -> f32 {
		match self {
			Self::Leaf(LeafNode {
				examples_fraction, ..
			}) => *examples_fraction,
			Self::Branch(BranchNode {
				examples_fraction, ..
			}) => *examples_fraction,
		}
	}
}

#[derive(Debug)]
pub struct BranchNode {
	pub left_child_index: usize,
	pub right_child_index: usize,
	pub split: BranchSplit,
	pub examples_fraction: f32,
}

#[derive(Debug)]
pub enum BranchSplit {
	Continuous(BranchSplitContinuous),
	Discrete(BranchSplitDiscrete),
}

impl BranchSplit {
	pub fn feature_index(&self) -> usize {
		match self {
			Self::Continuous(b) => b.feature_index,
			Self::Discrete(b) => b.feature_index,
		}
	}
}

#[derive(Debug)]
pub struct BranchSplitContinuous {
	pub feature_index: usize,
	pub split_value: f32,
	pub invalid_values_direction: SplitDirection,
}

#[derive(Clone, Debug)]
pub enum SplitDirection {
	Left,
	Right,
}

#[derive(Debug)]
pub struct BranchSplitDiscrete {
	pub feature_index: usize,
	pub directions: BinDirections,
}

#[derive(Clone, Debug)]
pub struct BinDirections {
	pub n: u8,
	pub bytes: [u8; 32],
}

#[derive(Debug)]
pub struct LeafNode {
	pub value: f32,
	pub examples_fraction: f32,
}
