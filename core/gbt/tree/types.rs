use super::{super::types, bin_stats::BinStats};
use std::cmp::Ordering;

#[derive(Debug)]
pub struct TrainTree {
	pub nodes: Vec<TrainNode>,
}

#[derive(Debug)]
pub enum TrainNode {
	Branch(TrainBranchNode),
	Leaf(TrainLeafNode),
}

#[derive(Debug)]
pub struct TrainBranchNode {
	pub left_child_index: Option<usize>,
	pub right_child_index: Option<usize>,
	pub split: TrainBranchSplit,
	pub missing_values_direction: types::SplitDirection,
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
	pub invalid_values_direction: types::SplitDirection,
}

#[derive(Clone, Debug)]
pub struct TrainBranchSplitDiscrete {
	pub feature_index: usize,
	pub directions: types::BinDirections,
	pub invalid_values_direction: types::SplitDirection,
}

#[derive(Debug)]
pub struct TrainLeafNode {
	pub value: f32,
	pub examples_fraction: f32,
}

pub struct QueueItem {
	/// Items in the priority queue will be sorted by the gain of the split
	pub gain: f32,
	pub split: TrainBranchSplit,

	/// The queue item holds a reference to its parent so that
	/// it can update the parent's left or right child index
	/// if the queue item becomes a node added to the tree.
	pub parent_index: Option<usize>,
	/// Will this node be a left or right child of its parent?
	pub split_direction: Option<types::SplitDirection>,

	pub depth: usize,
	pub bin_stats: BinStats,

	/// The examples_index_range tells you what the range of
	/// examples indexes in the examples_index specifies
	/// the examples in this node.
	pub examples_index_range: std::ops::Range<usize>,

	pub sum_gradients: f64,
	pub sum_hessians: f64,

	pub left_sum_gradients: f64,
	pub left_sum_hessians: f64,
	pub left_n_examples: usize,

	pub right_sum_gradients: f64,
	pub right_sum_hessians: f64,
	pub right_n_examples: usize,
}

impl From<TrainTree> for types::Tree {
	fn from(tree: TrainTree) -> Self {
		Self {
			nodes: tree.nodes.into_iter().map(Into::into).collect(),
		}
	}
}

impl From<TrainNode> for types::Node {
	fn from(node: TrainNode) -> Self {
		match node {
			TrainNode::Branch(TrainBranchNode {
				left_child_index,
				right_child_index,
				split,
				examples_fraction,
				..
			}) => types::Node::Branch(types::BranchNode {
				left_child_index: left_child_index.unwrap(),
				right_child_index: right_child_index.unwrap(),
				split: match split {
					TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
						feature_index,
						invalid_values_direction,
						split_value,
						..
					}) => types::BranchSplit::Continuous(types::BranchSplitContinuous {
						feature_index,
						split_value,
						invalid_values_direction,
					}),
					TrainBranchSplit::Discrete(TrainBranchSplitDiscrete {
						feature_index,
						directions,
						..
					}) => types::BranchSplit::Discrete(types::BranchSplitDiscrete {
						feature_index,
						directions,
					}),
				},
				examples_fraction,
			}),
			TrainNode::Leaf(TrainLeafNode {
				value,
				examples_fraction,
			}) => types::Node::Leaf(types::LeafNode {
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
