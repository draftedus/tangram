/*!
This crate is an implementation of machine learning models for regression and classification using ensembles of decision trees. The implementation has many similarities to [LightGBM](github.com/microsoft/lightgbm), [XGBoost](github.com/xgboost/xgboost), and many others, but written in pure Rust.

Through extensive optimization, `tangram_tree` is the fastest such implementation in our benchmarks, while achieving indistinguishable accuracy. All benchmarks are biased, and it is especially difficult to produce apples-to-apples comparisons because the algorithms to train decision trees are so complex and there are so ways to tweak them. However, we have done our best to choose widely accepted example datasets and set as few parameters away from defaults as possible.

## Benchmark results

Here's how to use `tangram_tree` to train a `Regressor`.

```
let mut features = tangram_dataframe::DataFrame::from_path("boston.csv", options, |_| {}).unwrap();
let labels = features.columns.remove(13);
let model = tangram_tree::Regressor::train(features, labels, Default::default(), &mut |_| {});
let features =
let mut predictions: Array1<f32> = Array::zeros(nrows);
model.predict(&features, &mut predictions);
```

There are three model types:

1. [`Regressor`](struct.Regressor.html)
2. [`BinaryClassifier`](struct.BinaryClassifier.html)
3. [`MulticlassClassifier`](struct.MulticlassClassifier.html)

The type of model you are going to train depends on the type of the column you want to predict and in the case of an enum column, the number of unique values.

| Label Type                                      | Number of Enum Options | Model Type                                                 |
|-------------------------------------------------|------------------------|------------------------------------------------------------|
| [Number](../dataframe/struct.NumberColumn.html) | *N/A*                  | [`Regressor`](struct.Regressor.html)                       |
| [Enum](../dataframe/struct.EnumColumn.html)     | 2                      | [`BinaryClassifier`](struct.BinaryClassifier.html)         |
| [Enum](../dataframe/struct.EnumColumn.html)     | > 2                    | [`MulticlassClassifier`](struct.MulticlassClassifier.html) |

Under the hood, tangram trees are Gradient Boosted Decision Trees.
*/

#![allow(clippy::tabs_in_doc_comments)]

mod bin;
mod bin_stats;
mod binary_classifier;
mod examples_index;
mod feature_importances;
mod multiclass_classifier;
mod regressor;
mod shap;
mod single;
mod split;
// mod timing;
mod train;

pub use binary_classifier::BinaryClassifier;
pub use multiclass_classifier::MulticlassClassifier;
pub use regressor::Regressor;

/// These are the options passed to `Regressor::train`, `BinaryClassifier::train`, and `MulticlassClassifier::train`.
#[derive(Debug)]
pub struct TrainOptions {
	/// If true, the model will include the loss on the training data at each round.
	pub compute_loss: bool,
	/// l2 regularization value to use for discrete splits.
	pub discrete_l2_regularization: f32,
	/// The minumum number of training examples that pass through this node for it to be considered for splitting.
	pub discrete_min_examples_per_branch: usize,
	/// Specify options for early stopping. If the value is `Some`, early stopping will be enabled. If it is `None`, early stopping will be disabled.
	pub early_stopping_options: Option<EarlyStoppingOptions>,
	/// L2 regularization helps avoid overfitting.
	pub l2_regularization: f32,
	/// The learning rate to use when computing the targets for the next tree.
	pub learning_rate: f32,
	/// The maximum depth we will grow a tree. Related to max_leaf_nodes. A fully dense tree will have a maximum of 2^depth leaf nodes.
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

/// This struct specifies the early stopping parameters that control what percentage of the dataset should be held out for early stopping, the number of early stopping rounds, and the threshold to determine when to stop training.
#[derive(Debug)]
pub struct EarlyStoppingOptions {
	/// the fraction of the dataset that we should set aside for use in early stopping
	pub early_stopping_fraction: f32,
	/// the maximum number of rounds of boosting that we will do if we don't see an improvement by at least `early_stopping_threshold` in the loss
	pub early_stopping_rounds: usize,
	/// This is the minimum amount a subsequent round of boosting must decrease the loss by. Early stopping can be thought of as a simple state machine: If we have a round that doesn't decrease the loss by at least tol, we increment our counter. If we decrease the loss by at least tol, the counter is reset to 0. If the counter hits early_stopping_rounds rounds, we stop training the tree.
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
			discrete_l2_regularization: 10.0,
			discrete_min_examples_per_branch: 100,
		}
	}
}

/// This struct reports the training progress.
#[derive(Debug)]
pub enum Progress {
	Initializing(tangram_progress::ProgressCounter),
	Training(tangram_progress::ProgressCounter),
}

/// A tree is described by a single vector of nodes.
#[derive(Debug)]
pub struct Tree {
	/// nodes in the tree
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

/// An enum describing the split direction, either `Left` or `Right`.
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
/// 	n: 4,
/// 	bytes: [2, ..]
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

/// Categorical splits are represented as a space-efficient bit vector.
/// If the entry at index i is 0, then the i-th enum variant goes to the left subtree
/// and if the value is 1, the i-th enum variant goes to the right subtree.
impl BinDirections {
	pub fn new(n: u8, value: bool) -> Self {
		let bytes = if !value { [0x00; 32] } else { [0xFF; 32] };
		Self { n, bytes }
	}

	/// Retrieves the bin direction for the enum variant given by `index`.
	/// Returns `None` if the index is greater than the total number of enum variants (n).
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

	/// Sets the bin direction for the given enum variant at `index` to the value passed, 0 if this enum variant should go the the left subtree and 1 if it should go to the right.
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
