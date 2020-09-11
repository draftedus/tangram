/*!
This module contains definitions of the crate's public types.
*/

use ndarray::prelude::*;

/// The options passed to tangram_tree::train
#[derive(Debug)]
pub struct TrainOptions {
	/// If true, the model will include the loss on the training data at each round.
	pub compute_loss: bool,
	/// l2 regularization value to use for discrete splits.
	pub discrete_l2_regularization: f32,
	/// TODO
	pub discrete_min_examples_per_branch: usize,
	/// TODO
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

/// This struct is used to specify the early stopping parameters that control what percentage of the dataset should be held out for early stopping, the number of early stopping rounds and the threshold to determine when to stop training.
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

/// This struct represents a tree regressor model. Regressor models are used to predict continuous target values, e.g. the selling price of a home.
#[derive(Debug)]
pub struct Regressor {
	/// The initial prediction of the model given no trained trees. The bias is calculated using the mean value of the target column in the training dataset.
	pub bias: f32,
	pub trees: Vec<Tree>,
	pub feature_importances: Option<Array1<f32>>,
	pub losses: Option<Array1<f32>>,
}

/// A Binary classifier model is trained to predict binary target values, e.g. does the patient have heart disease or not.
#[derive(Debug)]
pub struct BinaryClassifier {
	/// The initial prediction of the model given no trained trees. The bias is calculated using the distribution of the unique values in target column in the training dataset.
	pub bias: f32,
	/// The trees in this model.
	pub trees: Vec<Tree>,
	/// The feature importances of this model. These importances are computed using the ...TODO.
	pub feature_importances: Option<Array1<f32>>,
	/// The training losses in each round of training this model.
	pub losses: Option<Array1<f32>>,
	/// The names of the unique values in the target column.
	pub classes: Vec<String>,
}

/// This struct represents a tree multiclass classifier model. Multiclass classifier models are used to predict multiclass target values, e.g. species of flower is one of Iris Setosa, Iris Virginica, or Iris Versicolor.
#[derive(Debug)]
pub struct MulticlassClassifier {
	/// The initial prediction of the model given no trained trees. The bias is calculated using the distribution of the unique values in target column in the training dataset.
	pub biases: Vec<f32>,
	/// The trees in this model. It has shape (n_rounds, n_classes) because for each round, we train n_classes trees.
	pub trees: Vec<Tree>,
	// The number of classes.
	pub n_classes: usize,
	/// The number of boosting rounds == the number of trained trees.
	pub n_rounds: usize,
	/// TODO
	pub feature_importances: Option<Array1<f32>>,
	/// The training losses in each round of training this model.
	pub losses: Option<Array1<f32>>,
	/// The names of the unique values in the target column.
	pub classes: Vec<String>,
}

/// A Tree is described by a single vector of nodes.
#[derive(Debug)]
pub struct Tree {
	/// Nodes in the trained tree.
	pub nodes: Vec<Node>,
}

/** A Node represents the type of Node in the trained tree. It has two types:
1. **Branch**: A `BranchNode` represents internal tree nodes.
2. **Leaf**:  A `LeafNode` represents terminal nodes.
*/
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

/// A BranchNode describes an internal node in a trained tree.
///
#[derive(Debug)]
pub struct BranchNode {
	/// The index in the tree's node vector for this node's left child.
	pub left_child_index: usize,
	/// The index in the tree's node vector for this node's right child.
	pub right_child_index: usize,
	/// Used to determine how examples reaching this node should be routed, either to the left subtree or to the right.
	pub split: BranchSplit,
	/// The fraction of training examples that reach this node, used to compute SHAP values.
	pub examples_fraction: f32,
}

/// A BranchSplit describes how examples are routed to the left or right subtrees given their feature values. A BranchSplit is `Continous` if the best split for the node is for a numeric feature and `Discrete` if the best split of the node is for an enum feature.
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

/// This struct describes a continuous split used to determine how continuous numeric features are split into left/right subtrees.
#[derive(Debug)]
pub struct BranchSplitContinuous {
	/// The index of the feature used to split the node.
	pub feature_index: usize,
	/// The threshold value of the split.
	/// All features <= split_value go to the left subtree and all features  > split_value go to the right.
	pub split_value: f32,
	/// The subtree (left or right) that invalid values for this feature should go to.
	pub invalid_values_direction: SplitDirection,
}

#[derive(Clone, Debug)]
pub enum SplitDirection {
	Left,
	Right,
}

/// This struct describes a discrete split used to determine how enum features are split into left/right subtrees.
#[derive(Debug)]
pub struct BranchSplitDiscrete {
	/// The index of the feature used to split the node.
	pub feature_index: usize,
	/// The child node direction each enum variant belongs to, 0 for the left child and 1 for the right.
	pub directions: BinDirections,
}

/// This struct describes which subtree (left or right) a binned feature value should go to. It is a bitset where the bit value at index i represent which child the i-th enum variant should go: 0 for the left child and 1 for the right.
///
/// A feature whose value is the i-th enum variant should go to the left subtree if the i-th bit in the bitset is 0 and to the right subtree if the i-th bit is 1.
///
/// # Example
/// Consider an enum feature with three variants: `red`, `green`, and `blue`. We always reserve bin 0 for features with missing values.
/// ```
/// BinDirections {
///   n: 4,
///   bytes: [2, ..]
/// }
/// ```
/// We only need one byte to represent this feature since there are only 4 bins: 3 enum variants + 1 for the missing bin.
/// The first byte, represented as bits is `00000010`.
/// The enum variant 1, corresponding to `red` goes to the right subtree and `missing`, `blue` and `green` go to the left.
#[derive(Clone, Debug)]
pub struct BinDirections {
	/// The total number of bin directions in the bitset.
	pub n: u8,
	/// Bytes representing the direction (0=left and 1=right) for each bin.
	pub bytes: [u8; 32],
}

/// This struct describes a leaf node in a trained tree.
#[derive(Debug)]
pub struct LeafNode {
	/// The output of the leaf node... TODO
	pub value: f32,
	/// The fraction of the training examples that ended up in this leaf node, used to compute SHAP values.
	pub examples_fraction: f32,
}
