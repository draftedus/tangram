use super::{
	bin_stats::{
		compute_bin_stats_for_not_root, compute_bin_stats_for_root, compute_bin_stats_subtraction,
		BinStats,
	},
	binning::BinnedFeatures,
	examples_index::rearrange_examples_index,
	split::choose_best_split,
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
	#[cfg(feature = "debug")]
	pub depth: usize,
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
	binned_features: &BinnedFeatures,
	gradients: &[f32],
	hessians: &[f32],
	ordered_gradients: &mut [f32],
	ordered_hessians: &mut [f32],
	examples_index: &mut [i32],
	examples_index_left_buffer: &mut [i32],
	examples_index_right_buffer: &mut [i32],
	bin_stats_pool: &mut Pool<BinStats>,
	hessians_are_constant: bool,
	options: &TrainOptions,
	#[cfg(feature = "debug")] timing: &crate::timing::Timing,
) -> TrainTree {
	// These are the nodes in the tree returned by this function
	let mut nodes = Vec::new();
	// This priority queue stores the potential nodes to split ordered by their gain.
	let mut queue: BinaryHeap<QueueItem> = BinaryHeap::new();
	// To update the gradients and hessians we need to make predictions. Rather than running each example through the tree, we can reuse the mapping from example index to leaf value previously computed.
	let mut leaf_values: Vec<(Range<usize>, f32)> = Vec::new();

	// Compute the sums of gradients and hessians for the root node.
	#[cfg(feature = "debug")]
	let start = std::time::Instant::now();
	let n_examples_root = examples_index.len();
	let examples_index_range_root = 0..n_examples_root;
	let sum_gradients_root = gradients.iter().map(|v| v.to_f64().unwrap()).sum();
	let sum_hessians_root = if hessians_are_constant {
		n_examples_root.to_f64().unwrap()
	} else {
		hessians.iter().map(|v| v.to_f64().unwrap()).sum()
	};
	#[cfg(feature = "debug")]
	timing.sum_gradients_hessians.inc(start.elapsed());

	// Determine if we should try to split the root.
	let should_try_to_split_root = n_examples_root >= 2 * options.min_examples_per_node
		&& sum_hessians_root >= 2.0 * options.min_sum_hessians_per_node.to_f64().unwrap();
	if !should_try_to_split_root {
		add_leaf(
			&mut nodes,
			&mut leaf_values,
			sum_gradients_root,
			sum_hessians_root,
			n_examples_root,
			examples_index_range_root,
			#[cfg(feature = "debug")]
			0,
			None,
			options,
		);
		return TrainTree { nodes, leaf_values };
	}

	// Compute bin stats for the root node.
	#[cfg(feature = "debug")]
	let start = std::time::Instant::now();
	let mut root_bin_stats = bin_stats_pool.get().unwrap();
	compute_bin_stats_for_root(
		&mut root_bin_stats,
		binned_features,
		gradients,
		hessians,
		hessians_are_constant,
	);
	#[cfg(feature = "debug")]
	timing.compute_bin_stats_root.inc(start.elapsed());

	// Choose the best split for the root node.
	#[cfg(feature = "debug")]
	let start = std::time::Instant::now();
	let find_split_output = choose_best_split(
		&root_bin_stats,
		sum_gradients_root,
		sum_hessians_root,
		examples_index_range_root.clone(),
		&options,
	);
	#[cfg(feature = "debug")]
	timing.find_split.inc(start.elapsed());

	// If we were able to find a split for the root node, add it to the queue and proceed to the loop. Otherwise, return a tree with a single node.
	if let Some(find_split_output) = find_split_output {
		queue.push(QueueItem {
			gain: find_split_output.gain,
			parent_index: None,
			split_direction: None,
			depth: 0,
			bin_stats: root_bin_stats,
			examples_index_range: examples_index_range_root,
			sum_gradients: sum_gradients_root,
			sum_hessians: sum_hessians_root,
			split: find_split_output.split,
			left_n_examples: find_split_output.left_n_examples,
			left_sum_gradients: find_split_output.left_sum_gradients,
			left_sum_hessians: find_split_output.left_sum_hessians,
			right_n_examples: find_split_output.right_n_examples,
			right_sum_gradients: find_split_output.right_sum_gradients,
			right_sum_hessians: find_split_output.right_sum_hessians,
		});
	} else {
		add_leaf(
			&mut nodes,
			&mut leaf_values,
			sum_gradients_root,
			sum_hessians_root,
			n_examples_root,
			examples_index_range_root,
			#[cfg(feature = "debug")]
			0,
			None,
			options,
		);
		return TrainTree { nodes, leaf_values };
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
		#[cfg(feature = "debug")]
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
		let examples_index_range_start = queue_item.examples_index_range.start;
		let left_examples_index_range =
			examples_index_range_start + left.start..examples_index_range_start + left.end;
		let right_examples_index_range =
			examples_index_range_start + right.start..examples_index_range_start + right.end;
		#[cfg(feature = "debug")]
		timing.rearrange_examples_index.inc(start.elapsed());

		// Determine if we should try to split the left and/or right children of this branch.
		let children_will_exceed_max_depth = if let Some(max_depth) = options.max_depth {
			queue_item.depth + 1 > max_depth - 1
		} else {
			false
		};
		let should_try_to_split_left_child = !children_will_exceed_max_depth
			&& left_examples_index_range.len() >= options.min_examples_per_node * 2;
		let should_try_to_split_right_child = !children_will_exceed_max_depth
			&& right_examples_index_range.len() >= options.min_examples_per_node * 2;

		// If we should not split the left, add a leaf.
		if !should_try_to_split_left_child {
			add_leaf(
				&mut nodes,
				&mut leaf_values,
				queue_item.left_sum_gradients,
				queue_item.left_sum_hessians,
				queue_item.left_n_examples,
				left_examples_index_range.clone(),
				#[cfg(feature = "debug")]
				queue_item.depth,
				Some((node_index, SplitDirection::Left)),
				options,
			);
		}

		// If we should not split right, add a leaf.
		if !should_try_to_split_right_child {
			add_leaf(
				&mut nodes,
				&mut leaf_values,
				queue_item.right_sum_gradients,
				queue_item.right_sum_hessians,
				queue_item.right_n_examples,
				right_examples_index_range.clone(),
				#[cfg(feature = "debug")]
				queue_item.depth,
				Some((node_index, SplitDirection::Right)),
				options,
			);
		}

		// If we should not split either left or right, then there is nothing left to do, so we can go to the next item on the queue.
		if !should_try_to_split_left_child && !should_try_to_split_right_child {
			continue;
		}

		// Compute the bin stats for the two children. `smaller_child_direction` is the direction of the child to which fewer examples are sent.
		let smaller_child_direction =
			if left_examples_index_range.len() < right_examples_index_range.len() {
				SplitDirection::Left
			} else {
				SplitDirection::Right
			};
		let smaller_child_examples_index = match smaller_child_direction {
			SplitDirection::Left => &examples_index[left_examples_index_range.clone()],
			SplitDirection::Right => &examples_index[right_examples_index_range.clone()],
		};
		let mut smaller_child_bin_stats = bin_stats_pool.get().unwrap();

		// Compute the bin stats for the child with fewer examples.
		#[cfg(feature = "debug")]
		let start = std::time::Instant::now();
		compute_bin_stats_for_not_root(
			&mut smaller_child_bin_stats,
			ordered_gradients,
			ordered_hessians,
			binned_features,
			gradients,
			hessians,
			hessians_are_constant,
			smaller_child_examples_index,
		);
		#[cfg(feature = "debug")]
		timing.compute_bin_stats.inc(start.elapsed());

		// Compute the bin stats for the child with more examples by subtracting the bin stats of the child with fewer examples from the parent's bin stats.
		#[cfg(feature = "debug")]
		let start = std::time::Instant::now();
		let mut larger_child_bin_stats = queue_item.bin_stats;
		compute_bin_stats_subtraction(&mut larger_child_bin_stats, &smaller_child_bin_stats);
		#[cfg(feature = "debug")]
		timing.compute_bin_stats_subtraction.inc(start.elapsed());

		// Assign the smaller and larger bin stats to the left and right depending on which direction was smaller.
		let (left_bin_stats, right_bin_stats) = match smaller_child_direction {
			SplitDirection::Left => (smaller_child_bin_stats, larger_child_bin_stats),
			SplitDirection::Right => (larger_child_bin_stats, smaller_child_bin_stats),
		};

		// In order to do "best first" tree building, we need to choose the best split for each of the children of this branch.
		if should_try_to_split_left_child {
			#[cfg(feature = "debug")]
			let start = std::time::Instant::now();
			let left_find_split_output = choose_best_split(
				&left_bin_stats,
				queue_item.left_sum_gradients,
				queue_item.left_sum_hessians,
				left_examples_index_range.clone(),
				&options,
			);
			#[cfg(feature = "debug")]
			timing.find_split.inc(start.elapsed());
			if let Some(find_split_output) = left_find_split_output {
				queue.push(QueueItem {
					gain: find_split_output.gain,
					parent_index: Some(node_index),
					split_direction: Some(SplitDirection::Left),
					depth: queue_item.depth + 1,
					bin_stats: left_bin_stats,
					examples_index_range: left_examples_index_range.clone(),
					sum_gradients: queue_item.left_sum_gradients,
					sum_hessians: queue_item.left_sum_hessians,
					split: find_split_output.split,
					left_n_examples: find_split_output.left_n_examples,
					left_sum_gradients: find_split_output.left_sum_gradients,
					left_sum_hessians: find_split_output.left_sum_hessians,
					right_n_examples: find_split_output.right_n_examples,
					right_sum_gradients: find_split_output.right_sum_gradients,
					right_sum_hessians: find_split_output.right_sum_hessians,
				});
			} else {
				add_leaf(
					&mut nodes,
					&mut leaf_values,
					queue_item.left_sum_gradients,
					queue_item.left_sum_hessians,
					queue_item.left_n_examples,
					left_examples_index_range.clone(),
					#[cfg(feature = "debug")]
					queue_item.depth,
					Some((node_index, SplitDirection::Left)),
					options,
				);
			}
		}

		if should_try_to_split_right_child {
			#[cfg(feature = "debug")]
			let start = std::time::Instant::now();
			let right_find_split_output = choose_best_split(
				&right_bin_stats,
				queue_item.right_sum_gradients,
				queue_item.right_sum_hessians,
				right_examples_index_range.clone(),
				&options,
			);
			#[cfg(feature = "debug")]
			timing.find_split.inc(start.elapsed());
			if let Some(find_split_output) = right_find_split_output {
				queue.push(QueueItem {
					gain: find_split_output.gain,
					parent_index: Some(node_index),
					split_direction: Some(SplitDirection::Right),
					depth: queue_item.depth + 1,
					bin_stats: right_bin_stats,
					examples_index_range: right_examples_index_range.clone(),
					sum_gradients: queue_item.right_sum_gradients,
					sum_hessians: queue_item.right_sum_hessians,
					split: find_split_output.split,
					left_n_examples: find_split_output.left_n_examples,
					left_sum_gradients: find_split_output.left_sum_gradients,
					left_sum_hessians: find_split_output.left_sum_hessians,
					right_n_examples: find_split_output.right_n_examples,
					right_sum_gradients: find_split_output.right_sum_gradients,
					right_sum_hessians: find_split_output.right_sum_hessians,
				});
			} else {
				add_leaf(
					&mut nodes,
					&mut leaf_values,
					queue_item.right_sum_gradients,
					queue_item.right_sum_hessians,
					queue_item.right_n_examples,
					right_examples_index_range.clone(),
					#[cfg(feature = "debug")]
					queue_item.depth,
					Some((node_index, SplitDirection::Right)),
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
			queue_item.examples_index_range.len(),
			queue_item.examples_index_range,
			#[cfg(feature = "debug")]
			queue_item.depth,
			Some((
				queue_item.parent_index.unwrap(),
				queue_item.split_direction.unwrap(),
			)),
			options,
		);
	}

	TrainTree { nodes, leaf_values }
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
	#[cfg(feature = "debug")] depth: usize,
	parent_node_index_and_direction: Option<(usize, SplitDirection)>,
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
		#[cfg(feature = "debug")]
		depth,
	});
	leaf_values.push((examples_index_range, value));
	nodes.push(node);
	// Update the parent's left or right child index to refer to this leaf's index.
	if let Some((parent_node_index, parent_direction)) = parent_node_index_and_direction {
		let parent = nodes
			.get_mut(parent_node_index)
			.unwrap()
			.as_branch_mut()
			.unwrap();
		match parent_direction {
			SplitDirection::Left => parent.left_child_index = Some(leaf_index),
			SplitDirection::Right => parent.right_child_index = Some(leaf_index),
		}
	}
}
