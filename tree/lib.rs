/*!
This crate implements machine learning models for regression and classification using ensembles of decision trees. It has many similarities to [LightGBM](github.com/microsoft/lightgbm), [XGBoost](github.com/xgboost/xgboost), and others, but is written in pure Rust.

For an example of regression, see `benchmarks/boston.rs`.rs`. For an example of binary classification, see `benchmarks/heart-disease.rs`. For an example of multiclass classification, see `benchmarks/iris.rs`.
*/

#![allow(clippy::tabs_in_doc_comments)]

mod binary_classifier;
mod multiclass_classifier;
mod regressor;
mod shap;
#[cfg(feature = "timing")]
mod timing;
mod train;

pub use binary_classifier::BinaryClassifier;
pub use multiclass_classifier::MulticlassClassifier;
pub use regressor::Regressor;

/// These are the options passed to `Regressor::train`, `BinaryClassifier::train`, and `MulticlassClassifier::train`.
#[derive(Debug)]
pub struct TrainOptions {
	/// If true, the model will include the loss on the training data after each round.
	pub compute_loss: bool,
	/// This option controls early stopping. If it is `Some`, then early stopping will be enabled. If it is `None`, then early stopping will be disabled.
	pub early_stopping_options: Option<EarlyStoppingOptions>,
	/// This option sets the L2 regularization value, which helps avoid overfitting.
	pub l2_regularization: f32,
	/// The learning rate scales the leaf values to control the effect each tree has on the output.
	pub learning_rate: f32,
	/// The depth of a single tree will never exceed this value.
	pub max_depth: usize,
	/// The number of leaf nodes in a single tree will never exceed this value.
	pub max_leaf_nodes: usize,
	/// When computing the bin thresholds for number features, this is the maximum number of bins for valid values to create. If the number of unique values in the number feature is less than this value, the thresholds will be equal to the unique values, which can improve accuracy when number features have a small set of possible values.
	pub max_valid_bins_for_number_features: u8,
	/// This is the maximum number of rounds of training that will occur. Fewer rounds may be trained if early stopping is enabled.
	pub max_rounds: usize,
	/// A split will only be considered valid if the number of training examples sent to each of the resulting children is at least this value.
	pub min_examples_per_child: usize,
	/// A node will only be split if the best split achieves at least this minimum gain.
	pub min_gain_to_split: f32,
	/// A split will only be considered valid if the sum of hessians in each of the resulting children is at least this value.
	pub min_sum_hessians_per_child: f32,
	/// This is the maximum number of examples to consider when determining the bin thresholds for number features.
	pub max_examples_for_computing_bin_thresholds: usize,
	/// When choosing which direction each enum option should be sent in a discrete split, the enum options are sorted by a score computed from the sum of gradients and hessians for examples with that enum option. This smoothing factor is added to the denominator of that score.
	pub smoothing_factor_for_discrete_bin_sorting: f32,
	/// For discrete splits, the L2 regularization is `l2_regularization + supplemental_l2_regularization_for_discrete_splits`.
	pub supplemental_l2_regularization_for_discrete_splits: f32,
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
			max_rounds: 100,
			max_valid_bins_for_number_features: 255,
			min_examples_per_child: 20,
			min_gain_to_split: 0.0,
			min_sum_hessians_per_child: 1e-3,
			max_examples_for_computing_bin_thresholds: 200_000,
			smoothing_factor_for_discrete_bin_sorting: 10.0,
			supplemental_l2_regularization_for_discrete_splits: 10.0,
		}
	}
}

/// The parameters in this struct control how to determine whether training should stop early after each round. `early_stopping_fraction` is the fraction of the dataset that is set aside to compute the early stopping metric. If `early_stopping_rounds` rounds pass by without an improvement of at least `early_stopping_threshold` in the early stopping metric over the previous round, training will be stopped early.
#[derive(Debug)]
pub struct EarlyStoppingOptions {
	pub early_stopping_fraction: f32,
	pub early_stopping_rounds: usize,
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

impl Tree {
	/// Make a prediction for a given example.
	pub fn predict(&self, features: &[tangram_dataframe::Value]) -> f32 {
		// Start at the root node.
		let mut node_index = 0;
		// Traverse the tree until we get to a leaf.
		loop {
			match &self.nodes[node_index] {
				// This branch uses a continuous split.
				Node::Branch(BranchNode {
					left_child_index,
					right_child_index,
					split:
						BranchSplit::Continuous(BranchSplitContinuous {
							feature_index,
							split_value,
							..
						}),
					..
				}) => {
					node_index = if features[*feature_index].as_number().unwrap() <= split_value {
						*left_child_index
					} else {
						*right_child_index
					};
				}
				// This branch uses a discrete split.
				Node::Branch(BranchNode {
					left_child_index,
					right_child_index,
					split:
						BranchSplit::Discrete(BranchSplitDiscrete {
							feature_index,
							directions,
							..
						}),
					..
				}) => {
					let bin_index =
						if let Some(bin_index) = features[*feature_index].as_enum().unwrap() {
							bin_index.get()
						} else {
							0
						};
					node_index = if *directions.get(bin_index).unwrap() == SplitDirection::Left {
						*left_child_index
					} else {
						*right_child_index
					};
				}
				// We made it to a leaf! The prediction is the leaf's value.
				Node::Leaf(LeafNode { value, .. }) => return *value,
			}
		}
	}
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
	/// When making predictions, an example will be sent either to the right or left child. The `split` contains the information necessary to determine which way it will go.
	pub split: BranchSplit,
	/// Branch nodes store the fraction of training examples that passed through them during training. This is used to compute SHAP values.
	pub examples_fraction: f32,
}

/// A `BranchSplit` describes how examples are sent to the left or right child given their feature values. A `Continous` split is used for number features, and `Discrete` is used for enum features.
#[derive(Debug)]
pub enum BranchSplit {
	Continuous(BranchSplitContinuous),
	Discrete(BranchSplitDiscrete),
}

/// A continuous branch split takes the value of a single number feature, compares it with a `split_value`, and if the value is <= `split_value`, the example is sent left, and if it is > `split_value`, it is sent right.
#[derive(Debug)]
pub struct BranchSplitContinuous {
	/// This is the index of the feature to get the value for.
	pub feature_index: usize,
	/// This is the threshold value of the split.
	pub split_value: f32,
	/// This is the direction invalid values should be sent.
	pub invalid_values_direction: SplitDirection,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SplitDirection {
	Left,
	Right,
}

/// A discrete branch split takes the value of a single enum feature and looks up in a bitset which way the example should be sent.
#[derive(Debug)]
pub struct BranchSplitDiscrete {
	/// This is the index of the feature to get the value for.
	pub feature_index: usize,
	/// This specifies which direction, left or right, an example should be sent, based on the value of the chosen feature.
	pub directions: Vec<SplitDirection>,
}

/// The leaves in a tree hold the values to output for examples that get sent to them.
#[derive(Debug)]
pub struct LeafNode {
	/// This is the value to output.
	pub value: f32,
	/// Leaf nodes store the fraction of training examples that were sent to them during training. This is used to compute SHAP values.
	pub examples_fraction: f32,
}

impl BranchSplit {
	pub fn feature_index(&self) -> usize {
		match self {
			Self::Continuous(b) => b.feature_index,
			Self::Discrete(b) => b.feature_index,
		}
	}
}
