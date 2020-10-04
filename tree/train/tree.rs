use super::{
	bin_stats::BinStats,
	binning::{BinnedFeatures, BinningInstructions},
	examples_index::rearrange_examples_index,
	split::{
		choose_best_split_root, choose_best_splits_not_root, ChooseBestSplitOutput,
		ChooseBestSplitSuccess,
	},
};
use crate::{SplitDirection, TrainOptions};
use num_traits::ToPrimitive;
use std::{cmp::Ordering, collections::BinaryHeap, ops::Range};
use tangram_pool::{Pool, PoolGuard};

#[derive(Debug)]
pub struct TrainTree {
	pub nodes: Vec<TrainNode>,
	pub leaf_values: Vec<(Range<usize>, f32)>,
}

impl TrainTree {
	/// Make a prediction.
	pub fn predict(&self, example: &[tangram_dataframe::Value]) -> f32 {
		// Start at the root node.
		let mut node_index = 0;
		// Traverse the tree until we get to a leaf.
		loop {
			match &self.nodes.get(node_index).unwrap() {
				// This branch uses a continuous split.
				TrainNode::Branch(TrainBranchNode {
					left_child_index,
					right_child_index,
					split:
						TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
							feature_index,
							split_value,
							..
						}),
					..
				}) => {
					node_index = if example[*feature_index].as_number().unwrap() <= split_value {
						left_child_index.unwrap()
					} else {
						right_child_index.unwrap()
					};
				}
				// This branch uses a discrete split.
				TrainNode::Branch(TrainBranchNode {
					left_child_index,
					right_child_index,
					split:
						TrainBranchSplit::Discrete(TrainBranchSplitDiscrete {
							feature_index,
							directions,
							..
						}),
					..
				}) => {
					let bin_index =
						if let Some(bin_index) = example[*feature_index].as_enum().unwrap() {
							bin_index.get()
						} else {
							0
						};
					node_index = if *directions.get(bin_index).unwrap() == SplitDirection::Left {
						left_child_index.unwrap()
					} else {
						right_child_index.unwrap()
					};
				}
				// We made it to a leaf! The prediction is the leaf's value.
				TrainNode::Leaf(TrainLeafNode { value, .. }) => return *value,
			}
		}
	}
}

#[derive(Debug)]
pub enum TrainNode {
	Branch(TrainBranchNode),
	Leaf(TrainLeafNode),
}

impl TrainNode {
	pub fn as_branch_mut(&mut self) -> Option<&mut TrainBranchNode> {
		match self {
			TrainNode::Branch(s) => Some(s),
			_ => None,
		}
	}
}

#[derive(Debug)]
pub struct TrainBranchNode {
	pub left_child_index: Option<usize>,
	pub right_child_index: Option<usize>,
	pub split: TrainBranchSplit,
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
	pub bin_index: usize,
	pub invalid_values_direction: SplitDirection,
}

#[derive(Clone, Debug)]
pub struct TrainBranchSplitDiscrete {
	pub feature_index: usize,
	pub directions: Vec<SplitDirection>,
}

#[derive(Debug)]
pub struct TrainLeafNode {
	pub value: f32,
	pub examples_fraction: f32,
}

struct QueueItem {
	/// The priority queue will be sorted by the gain of the split.
	pub gain: f32,
	/// The queue item holds a reference to its parent so that it can update the parent's left or right child index if the queue item becomes a node added to the tree.
	pub parent_index: Option<usize>,
	/// Will this node be a left or right child of its parent?
	pub split_direction: Option<SplitDirection>,
	/// This is the depth of the item in the tree.
	pub depth: usize,
	/// The bin_stats consisting of aggregate hessian/gradient statistics of the training examples that reach this node.
	pub bin_stats: PoolGuard<BinStats>,
	/// The examples_index_range tells you what range of entries in the examples index correspond to this node.
	pub examples_index_range: std::ops::Range<usize>,
	/// This is the sum of the gradients for the training examples that pass through this node.
	pub sum_gradients: f64,
	/// This is the sum of the hessians for the training examples that pass through this node.
	pub sum_hessians: f64,
	/// This is the best split that was chosen for this node.
	pub split: TrainBranchSplit,
	/// This is the number of training examples that were sent to the left child.
	pub left_n_examples: usize,
	/// This is the sum of the gradients for the training examples that were sent to the left child.
	pub left_sum_gradients: f64,
	/// This is the sum of the hessians for the training examples that were sent to the left child.
	pub left_sum_hessians: f64,
	/// This is the number of training examples that were sent to the right child.
	pub right_n_examples: usize,
	/// This is the sum of the gradients for the training examples that were sent to the right child.
	pub right_sum_gradients: f64,
	/// This is the sum of the hessians for the training examples that were sent to the right child.
	pub right_sum_hessians: f64,
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

/// Train a tree.
#[allow(clippy::too_many_arguments)]
pub fn train(
	binning_instructions: &[BinningInstructions],
	binned_features: &BinnedFeatures,
	gradients: &[f32],
	hessians: &[f32],
	gradients_ordered_buffer: &mut [f32],
	hessians_ordered_buffer: &mut [f32],
	examples_index: &mut [i32],
	examples_index_left_buffer: &mut [i32],
	examples_index_right_buffer: &mut [i32],
	bin_stats_pool: &mut Pool<BinStats>,
	hessians_are_constant: bool,
	options: &TrainOptions,
	#[cfg(feature = "timing")] timing: &crate::timing::Timing,
) -> TrainTree {
	// These are the nodes in the tree returned by this function
	let mut nodes = Vec::new();
	// This priority queue stores the potential nodes to split ordered by their gain.
	let mut queue: BinaryHeap<QueueItem> = BinaryHeap::new();
	// To update the gradients and hessians we need to make predictions. Rather than running each example through the tree, we can reuse the mapping from example index to leaf value previously computed.
	let mut leaf_values: Vec<(Range<usize>, f32)> = Vec::new();

	let n_examples_root = examples_index.len();
	let examples_index_range_root = 0..n_examples_root;

	// Choose the best split for the root node.
	let choose_best_split_output_root = choose_best_split_root(
		bin_stats_pool,
		binning_instructions,
		binned_features,
		gradients,
		hessians,
		hessians_are_constant,
		options,
		#[cfg(feature = "timing")]
		&timing,
	);

	// If we were able to find a split for the root node, add it to the queue and proceed to the loop. Otherwise, return a tree with a single node.
	match choose_best_split_output_root {
		ChooseBestSplitOutput::Success(output) => {
			add_queue_item(&mut queue, output, None, None, 0, examples_index_range_root);
		}
		ChooseBestSplitOutput::Failure(output) => {
			add_leaf(
				&mut nodes,
				&mut leaf_values,
				output.sum_gradients,
				output.sum_hessians,
				n_examples_root,
				examples_index_range_root,
				None,
				None,
				options,
			);
			return TrainTree { nodes, leaf_values };
		}
	}

	// This is the training loop for a tree.
	loop {
		// If we will hit the maximum number of leaf nodes by adding the remaining queue items as leaves then exit the loop.
		let n_leaf_nodes = leaf_values.len() + queue.len();
		let max_leaf_nodes_reached = n_leaf_nodes == options.max_leaf_nodes;
		if max_leaf_nodes_reached {
			break;
		}

		// Pop an item off the queue.
		let node_index = nodes.len();
		let queue_item = if let Some(queue_item) = queue.pop() {
			queue_item
		} else {
			break;
		};

		// Create the new branch node.
		let examples_fraction = queue_item.examples_index_range.len().to_f32().unwrap()
			/ n_examples_root.to_f32().unwrap();
		nodes.push(TrainNode::Branch(TrainBranchNode {
			split: queue_item.split.clone(),
			left_child_index: None,
			right_child_index: None,
			examples_fraction,
		}));
		if let Some(parent_index) = queue_item.parent_index {
			let parent = nodes
				.get_mut(parent_index)
				.unwrap()
				.as_branch_mut()
				.unwrap();
			let split_direction = queue_item.split_direction.unwrap();
			match split_direction {
				SplitDirection::Left => parent.left_child_index = Some(node_index),
				SplitDirection::Right => parent.right_child_index = Some(node_index),
			}
		}

		// Rearrange the examples index.
		#[cfg(feature = "timing")]
		let start = std::time::Instant::now();
		let (left, right) = rearrange_examples_index(
			binned_features,
			&queue_item.split,
			examples_index
				.get_mut(queue_item.examples_index_range.clone())
				.unwrap(),
			examples_index_left_buffer
				.get_mut(queue_item.examples_index_range.clone())
				.unwrap(),
			examples_index_right_buffer
				.get_mut(queue_item.examples_index_range.clone())
				.unwrap(),
		);
		// The left and right ranges are local to the node, so add the node's start to make them global.
		let branch_examples_index_range_start = queue_item.examples_index_range.start;
		let left_child_examples_index_range = branch_examples_index_range_start + left.start
			..branch_examples_index_range_start + left.end;
		let right_child_examples_index_range = branch_examples_index_range_start + right.start
			..branch_examples_index_range_start + right.end;
		let left_child_examples_index = examples_index
			.get(left_child_examples_index_range.clone())
			.unwrap();
		let right_child_examples_index = examples_index
			.get(right_child_examples_index_range.clone())
			.unwrap();
		#[cfg(feature = "timing")]
		timing.rearrange_examples_index.inc(start.elapsed());

		// Choose the best splits for each of the right and left children of this new branch.
		let (left_child_best_split_output, right_child_best_split_output) =
			choose_best_splits_not_root(
				bin_stats_pool,
				binning_instructions,
				binned_features,
				queue_item.depth,
				gradients,
				hessians,
				queue_item.left_n_examples,
				queue_item.left_sum_gradients,
				queue_item.left_sum_hessians,
				queue_item.right_n_examples,
				queue_item.right_sum_gradients,
				queue_item.right_sum_hessians,
				left_child_examples_index,
				right_child_examples_index,
				gradients_ordered_buffer,
				hessians_ordered_buffer,
				queue_item.bin_stats,
				hessians_are_constant,
				options,
				#[cfg(feature = "timing")]
				timing,
			);

		// Add a queue item or leaf for the left child.
		match left_child_best_split_output {
			ChooseBestSplitOutput::Success(output) => {
				add_queue_item(
					&mut queue,
					output,
					Some(node_index),
					Some(SplitDirection::Left),
					queue_item.depth + 1,
					left_child_examples_index_range,
				);
			}
			ChooseBestSplitOutput::Failure(output) => {
				add_leaf(
					&mut nodes,
					&mut leaf_values,
					output.sum_gradients,
					output.sum_hessians,
					n_examples_root,
					left_child_examples_index_range,
					Some(node_index),
					Some(SplitDirection::Left),
					options,
				);
			}
		}

		// Add a queue item or leaf for the right child.
		match right_child_best_split_output {
			ChooseBestSplitOutput::Success(output) => {
				add_queue_item(
					&mut queue,
					output,
					Some(node_index),
					Some(SplitDirection::Right),
					queue_item.depth + 1,
					right_child_examples_index_range,
				);
			}
			ChooseBestSplitOutput::Failure(output) => {
				add_leaf(
					&mut nodes,
					&mut leaf_values,
					output.sum_gradients,
					output.sum_hessians,
					n_examples_root,
					right_child_examples_index_range,
					Some(node_index),
					Some(SplitDirection::Right),
					options,
				);
			}
		}
	}

	// The remaining items on the queue should all be made into leaves.
	while let Some(queue_item) = queue.pop() {
		add_leaf(
			&mut nodes,
			&mut leaf_values,
			queue_item.sum_gradients,
			queue_item.sum_hessians,
			n_examples_root,
			queue_item.examples_index_range,
			Some(queue_item.parent_index.unwrap()),
			Some(queue_item.split_direction.unwrap()),
			options,
		);
	}

	TrainTree { nodes, leaf_values }
}

/// Add a queue item to the queue.
fn add_queue_item(
	queue: &mut BinaryHeap<QueueItem>,
	output: ChooseBestSplitSuccess,
	parent_index: Option<usize>,
	split_direction: Option<SplitDirection>,
	depth: usize,
	examples_index_range: Range<usize>,
) {
	queue.push(QueueItem {
		gain: output.gain,
		parent_index,
		split_direction,
		depth,
		bin_stats: output.bin_stats,
		examples_index_range,
		sum_gradients: output.sum_gradients,
		sum_hessians: output.sum_hessians,
		split: output.split,
		left_n_examples: output.left_n_examples,
		left_sum_gradients: output.left_sum_gradients,
		left_sum_hessians: output.left_sum_hessians,
		right_n_examples: output.right_n_examples,
		right_sum_gradients: output.right_sum_gradients,
		right_sum_hessians: output.right_sum_hessians,
	});
}

/// Add a leaf to the list of nodes and update the parent to refer to it.
#[allow(clippy::too_many_arguments)]
fn add_leaf(
	nodes: &mut Vec<TrainNode>,
	leaf_values: &mut Vec<(Range<usize>, f32)>,
	sum_gradients: f64,
	sum_hessians: f64,
	n_examples_root: usize,
	examples_index_range: Range<usize>,
	parent_node_index: Option<usize>,
	split_direction: Option<SplitDirection>,
	options: &TrainOptions,
) {
	// This is the index this leaf will have in the `nodes` array.
	let leaf_index = nodes.len();
	// Compute the leaf's value.
	let value = (-options.learning_rate.to_f64().unwrap() * sum_gradients
		/ (sum_hessians + options.l2_regularization.to_f64().unwrap() + std::f64::EPSILON))
		.to_f32()
		.unwrap();
	let examples_fraction =
		examples_index_range.len().to_f32().unwrap() / n_examples_root.to_f32().unwrap();
	let node = TrainNode::Leaf(TrainLeafNode {
		value,
		examples_fraction,
	});
	leaf_values.push((examples_index_range, value));
	nodes.push(node);
	// Update the parent's left or right child index to refer to this leaf's index.
	if let Some(parent_node_index) = parent_node_index {
		let parent = nodes
			.get_mut(parent_node_index)
			.unwrap()
			.as_branch_mut()
			.unwrap();
		match split_direction.unwrap() {
			SplitDirection::Left => parent.left_child_index = Some(leaf_index),
			SplitDirection::Right => parent.right_child_index = Some(leaf_index),
		}
	}
}
