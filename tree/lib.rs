/*!
This crate implements machine learning models for regression and classification using ensembles of decision trees. It has many similarities to [LightGBM](github.com/microsoft/lightgbm), [XGBoost](github.com/xgboost/xgboost), and others, but is written in pure Rust.

For regression example, see `benchmarks/boston.rs`.rs`.
For binary classification examples, see `benchmarks/census.rs` and `benchmakrs/boston.rs`.
For a multiclass classification example, see `benchmarks/iris.rs`.
*/

#![allow(clippy::tabs_in_doc_comments)]

mod binary_classifier;
mod multiclass_classifier;
mod regressor;
mod shap;
mod single;
#[cfg(feature = "timing")]
mod timing;
mod train;

pub use binary_classifier::BinaryClassifier;
pub use multiclass_classifier::MulticlassClassifier;
pub use regressor::Regressor;

/// These are the options passed to `Regressor::train`, `BinaryClassifier::train`, and `MulticlassClassifier::train`.
#[derive(Debug)]
pub struct TrainOptions {
	/// If true, the model will include the loss on the training data at each round.
	pub compute_loss: bool,
	/// This is the L2 regularization value to use for discrete splits.
	pub discrete_l2_regularization: f32,
	/// This is the minumum number of training examples that pass through this node for it to be considered for splitting.
	pub discrete_min_examples_per_branch: usize,
	/// Specify options for early stopping. If the value is `Some`, early stopping will be enabled. If it is `None`, early stopping will be disabled.
	pub early_stopping_options: Option<EarlyStoppingOptions>,
	/// L2 regularization helps avoid overfitting.
	pub l2_regularization: f32,
	/// This is the learning rate to use when computing the targets for the next tree.
	pub learning_rate: f32,
	/// This is the maximum depth we will grow a tree. Related to max_leaf_nodes. A fully dense tree will have a maximum of 2^depth leaf nodes.
	pub max_depth: usize,
	/// This is the maximum number of leaf nodes before stopping to train an individual tree.
	pub max_leaf_nodes: usize,
	/// This is the maximum number of bins to use when mapping our feature values into binned features. The default value is 255 because we want to store binned indexes as u8. Follows the convention in sklearn, lightgbm and xgboost. The final bin (256) is reserved for missing values.
	pub max_non_missing_bins: u8,
	/// This is the maximum number of rounds of boosting, could be less if we are using early stopping.  The number of trees is related to the the max_rounds. In regression and binary classification, the maximum number of trees is equal to max rounds. In multiclass classification, the maximum number of trees is num_classes * max_rounds. >= 0. If max_rounds = 0, the baseline classifier is returned, (just the bias).
	pub max_rounds: usize,
	/// This is the minimum number of examples that must be present in a leaf node during training in order to add the node to the tree.
	pub min_examples_per_leaf: usize,
	/// This is the minimum gain for a node to become a branch instead of a leaf.
	pub min_gain_to_split: f32,
	/// The minimum value of the sum of hessians for a node to become a branch instead of a leaf.
	pub min_sum_hessians_in_leaf: f32,
	/// The maximum number of examples to consider for determining the bin thresholds for number columns.
	pub subsample_for_binning: usize,
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
			min_examples_per_leaf: 20,
			min_sum_hessians_in_leaf: 1e-3,
			min_gain_to_split: 0.0,
			discrete_l2_regularization: 10.0,
			discrete_min_examples_per_branch: 20,
		}
	}
}

/// The parameters in this struct control how to determine whether training should stop early after each round.
#[derive(Debug)]
pub struct EarlyStoppingOptions {
	/// This is the fraction of the dataset that is set aside to compute the early stopping metric.
	pub early_stopping_fraction: f32,
	/// If this many rounds pass by without a significant improvement in the early stopping metric over the previous round, training will be stopped early.
	pub early_stopping_rounds: usize,
	/// This is the minimum descrease in the early stopping metric for a round to be considered a significant improvement over the previous round.
	pub early_stopping_threshold: f32,
}

/// This struct reports the training progress.
#[derive(Debug)]
pub enum TrainProgress {
	Initializing(tangram_progress::ProgressCounter),
	Training(tangram_progress::ProgressCounter),
}

/// Trees are stored as a `Vec` of `Node`s. Each branch in the tree has two indexes into the `Vec`, one for each of its children.
#[derive(Debug)]
pub struct Tree {
	pub nodes: Vec<Node>,
}

/// A node is either a branch or a leaf.
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

/// A `BranchNode` is a branch in a tree.
#[derive(Debug)]
pub struct BranchNode {
	/// This is the index in the tree's node vector for this node's left child.
	pub left_child_index: usize,
	/// This is the index in the tree's node vector for this node's right child.
	pub right_child_index: usize,
	/// In prediction, an example will be sent either to the right or left child. The `split` contains the information necessary to determine which way it will go.
	pub split: BranchSplit,
	/// Branch nodes store the fraction of training examples that passed through them during training. This is used to compute SHAP values.
	pub examples_fraction: f32,
}

/// A `BranchSplit` describes how examples are sent to the left or right child given their feature values. A `Continous` split is used for `Number` features, and `Discrete` is used for `Enum` features.
#[derive(Debug)]
pub enum BranchSplit {
	Continuous(BranchSplitContinuous),
	Discrete(BranchSplitDiscrete),
}

/// A continuous branch split takes the value of a single `Number` feature, compares it with a `split_value`, and if the value is <= `split_value`, the example is sent left, and if it is > `split_value`, it is sent right.
#[derive(Debug)]
pub struct BranchSplitContinuous {
	/// This is the index of the feature to get the value for.
	pub feature_index: usize,
	/// This is the threshold value of the split.
	pub split_value: f32,
	/// Which direction should invalid values go?
	pub invalid_values_direction: SplitDirection,
}

#[derive(Clone, Debug)]
pub enum SplitDirection {
	Left,
	Right,
}

/// A discrete branch split takes the value of a single `Enum` feature and looks up in a bit set which way the example should be sent.
#[derive(Debug)]
pub struct BranchSplitDiscrete {
	/// This is the index of the feature to get the value for.
	pub feature_index: usize,
	/// `directions` specifies which direction, left or right, an example should be sent, based on the value of the chosen feature.
	pub directions: BinDirections,
}

/// The leaves in a tree hold the values to output for examples that get sent to them.
#[derive(Debug)]
pub struct LeafNode {
	/// This is the value to output.
	pub value: f32,
	/// Leaf nodes store the fraction of training examples that were sent to them during training. This is used to compute SHAP values.
	pub examples_fraction: f32,
}

/**
`BinDirections` specifies which direction, left or right, an example should be sent, based on the value of an `Enum` feature. Just like `Enum` features, bin 0 is reserved for invalid values. Rather than use a Vec<bool>, to avoid heap allocation and minimize the size of the struct, we use a bitset.
*/
#[derive(Clone, Debug)]
pub struct BinDirections {
	/// The total number of bin directions in the bitset.
	pub n: u8,
	/// Bytes representing the direction (0=left and 1=right) for each bin.
	pub bytes: [u8; 32],
}

impl BinDirections {
	pub fn new(n: u8, value: bool) -> Self {
		let bytes = if !value { [0x00; 32] } else { [0xFF; 32] };
		Self { n, bytes }
	}

	/// Retrieve the bin direction for the enum variant given by `index`. This will return `None` if the index is greater than the total number of enum variants (n).
	pub fn get(&self, index: u8) -> Option<bool> {
		if index >= self.n {
			None
		} else {
			let byte_index = (index / 8) as usize;
			let byte = self.bytes[byte_index];
			let bit_index = index % 8;
			let bit = (byte >> bit_index) & 0b0000_0001;
			Some(bit == 1)
		}
	}

	/// Set the bin direction for the given enum variant at `index` to the value passed, 0 if this enum variant should go the the left subtree and 1 if it should go to the right.
	pub fn set(&mut self, index: u8, value: bool) {
		let byte_index = (index / 8) as usize;
		let bit_index = index % 8;
		if value {
			self.bytes[byte_index] |= 1 << bit_index;
		} else {
			self.bytes[byte_index] &= !(1 << bit_index);
		}
	}
}

impl BranchSplit {
	pub fn feature_index(&self) -> usize {
		match self {
			Self::Continuous(b) => b.feature_index,
			Self::Discrete(b) => b.feature_index,
		}
	}
}
