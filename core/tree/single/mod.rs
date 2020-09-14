use self::bin_stats::BinStats;
use super::{
	BinDirections, BranchNode, BranchSplit, BranchSplitContinuous, BranchSplitDiscrete, LeafNode,
	Node, SplitDirection, Tree,
};
use crate::tree;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::cmp::Ordering;

pub mod bin_directions;
pub mod bin_stats;
mod examples_index;
pub mod split;
pub mod train;

/// A TrainTree is described by a single vector of nodes.
#[derive(Debug)]
pub struct TrainTree {
	pub nodes: Vec<TrainNode>,
}

/** A TrainNode represents the type of TrainNode in the tree. It has two types:
1. **Branch**: A `BranchNode` represents internal tree nodes.
2. **Leaf**:  A `LeafNode` represents terminal nodes.
*/
#[derive(Debug)]
pub enum TrainNode {
	Branch(TrainBranchNode),
	Leaf(TrainLeafNode),
}

#[derive(Debug)]
pub struct TrainBranchNode {
	/// The index in the TrainTree's nodes of this node's left child
	pub left_child_index: Option<usize>,
	/// The index in the TrainTree's nodes of this node's right child
	pub right_child_index: Option<usize>,
	/// The best split for this node.
	pub split: TrainBranchSplit,
	/// Missing values direction specifies whether examples whose feature value is missing should go to the left subtree or the right.
	pub missing_values_direction: SplitDirection,
	/// The fraction of the total training examples that reach this node.
	pub examples_fraction: f32,
}

#[derive(Clone, Debug)]
pub enum TrainBranchSplit {
	Continuous(TrainBranchSplitContinuous),
	Discrete(TrainBranchSplitDiscrete),
}

#[derive(Clone, Debug)]
pub struct TrainBranchSplitContinuous {
	pub feature_index: usize,
	pub split_value: f32,
	pub bin_index: u8,
	pub invalid_values_direction: SplitDirection,
}

#[derive(Clone, Debug)]
pub struct TrainBranchSplitDiscrete {
	pub feature_index: usize,
	pub directions: BinDirections,
	pub invalid_values_direction: SplitDirection,
}

#[derive(Debug)]
pub struct TrainLeafNode {
	pub value: f32,
	pub examples_fraction: f32,
}

pub struct QueueItem {
	/// Items in the priority queue will be sorted by the gain of the split.
	pub gain: f32,
	/// A split describes how the node is split into left and right children.
	pub split: TrainBranchSplit,

	/// The queue item holds a reference to its parent so that
	/// it can update the parent's left or right child index
	/// if the queue item becomes a node added to the tree.
	pub parent_index: Option<usize>,
	/// Will this node be a left or right child of its parent?
	pub split_direction: Option<SplitDirection>,

	/// The depth of the item in the tree.
	pub depth: usize,
	/// The bin_stats consisting of aggregate hessian/gradient statistics of the training examples that reach this node.
	pub bin_stats: BinStats,

	/// The examples_index_range tells you what the range of
	/// examples indexes in the examples_index specifies
	/// the examples in this node.
	pub examples_index_range: std::ops::Range<usize>,

	/// The sum of the gradients of all of the training examples in this node.
	pub sum_gradients: f64,
	/// The sum of the hessians of all of the training examples in this node.
	pub sum_hessians: f64,

	/// The sum of the gradients of all of the training examples that go to the left child.
	pub left_sum_gradients: f64,
	/// The sum of the hessians of all of the training examples that go to the left child.
	pub left_sum_hessians: f64,
	/// The total number of training examples that go to the left child.
	pub left_n_examples: usize,

	/// The sum of the gradients of all of the training examples that go to the right child.
	pub right_sum_gradients: f64,
	/// The sum of the hessians of all of the training examples that go to the right child.
	pub right_sum_hessians: f64,
	/// The total number of training examples that go to the right child.
	pub right_n_examples: usize,
}

impl From<TrainTree> for Tree {
	fn from(tree: TrainTree) -> Self {
		Self {
			nodes: tree.nodes.into_iter().map(Into::into).collect(),
		}
	}
}

impl From<TrainNode> for Node {
	fn from(node: TrainNode) -> Self {
		match node {
			TrainNode::Branch(TrainBranchNode {
				left_child_index,
				right_child_index,
				split,
				examples_fraction,
				..
			}) => Node::Branch(BranchNode {
				left_child_index: left_child_index.unwrap(),
				right_child_index: right_child_index.unwrap(),
				split: match split {
					TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
						feature_index,
						invalid_values_direction,
						split_value,
						..
					}) => BranchSplit::Continuous(BranchSplitContinuous {
						feature_index,
						split_value,
						invalid_values_direction,
					}),
					TrainBranchSplit::Discrete(TrainBranchSplitDiscrete {
						feature_index,
						directions,
						..
					}) => BranchSplit::Discrete(BranchSplitDiscrete {
						feature_index,
						directions,
					}),
				},
				examples_fraction,
			}),
			TrainNode::Leaf(TrainLeafNode {
				value,
				examples_fraction,
			}) => Node::Leaf(LeafNode {
				value,
				examples_fraction,
			}),
		}
	}
}

impl PartialEq for QueueItem {
	fn eq(&self, other: &Self) -> bool {
		self.gain == other.gain
	}
}

impl Eq for QueueItem {}

impl std::cmp::PartialOrd for QueueItem {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.gain.partial_cmp(&other.gain)
	}
}

impl std::cmp::Ord for QueueItem {
	fn cmp(&self, other: &Self) -> Ordering {
		self.partial_cmp(&other).unwrap()
	}
}

impl TrainNode {
	pub fn as_branch_mut(&mut self) -> Option<&mut TrainBranchNode> {
		match self {
			TrainNode::Branch(s) => Some(s),
			_ => None,
		}
	}
}

impl tree::Tree {
	pub fn predict(&self, row: &[tangram_dataframe::Value]) -> f32 {
		let mut node_index = 0;
		loop {
			match &self.nodes[node_index] {
				tree::Node::Branch(tree::BranchNode {
					left_child_index,
					right_child_index,
					split:
						tree::BranchSplit::Continuous(tree::BranchSplitContinuous {
							feature_index,
							split_value,
							invalid_values_direction,
							..
						}),
					..
				}) => {
					let feature_value = match row[*feature_index] {
						tangram_dataframe::Value::Number(value) => value,
						_ => unreachable!(),
					};
					node_index = if feature_value.is_nan() {
						match invalid_values_direction {
							tree::SplitDirection::Left => *left_child_index,
							tree::SplitDirection::Right => *right_child_index,
						}
					} else if feature_value <= *split_value {
						*left_child_index
					} else {
						*right_child_index
					};
				}
				tree::Node::Branch(tree::BranchNode {
					left_child_index,
					right_child_index,
					split:
						tree::BranchSplit::Discrete(tree::BranchSplitDiscrete {
							feature_index,
							directions,
							..
						}),
					..
				}) => {
					let feature_value = match row[*feature_index] {
						tangram_dataframe::Value::Enum(value) => value.to_u8().unwrap(),
						_ => unreachable!(),
					};
					node_index = if !directions.get(feature_value).unwrap() {
						*left_child_index
					} else {
						*right_child_index
					};
				}
				tree::Node::Leaf(tree::LeafNode { value, .. }) => return *value,
			}
		}
	}
}

impl tree::single::TrainTree {
	pub fn predict(&self, features: ArrayView1<u8>) -> f32 {
		let mut node_index = 0;
		loop {
			match &self.nodes[node_index] {
				tree::single::TrainNode::Branch(tree::single::TrainBranchNode {
					left_child_index,
					right_child_index,
					split,
					..
				}) => match split {
					tree::single::TrainBranchSplit::Continuous(
						tree::single::TrainBranchSplitContinuous {
							feature_index,
							bin_index,
							..
						},
					) => {
						node_index = if features[*feature_index] <= *bin_index {
							left_child_index.unwrap()
						} else {
							right_child_index.unwrap()
						};
					}
					tree::single::TrainBranchSplit::Discrete(
						tree::single::TrainBranchSplitDiscrete {
							feature_index,
							directions,
							..
						},
					) => {
						let bin_index = features[*feature_index];
						node_index = if !directions.get(bin_index).unwrap() {
							left_child_index.unwrap()
						} else {
							right_child_index.unwrap()
						};
					}
				},
				tree::single::TrainNode::Leaf(tree::single::TrainLeafNode { value, .. }) => {
					return *value
				}
			}
		}
	}
}
