use super::{
	super::{timing, types},
	bin_stats::{
		compute_bin_stats_for_non_root_node, compute_bin_stats_for_root_node,
		compute_bin_stats_subtraction, BinStatsPool,
	},
	examples_index::rearrange_examples_index,
	split::{find_split, find_split_both},
	types::*,
};
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use rayon::prelude::*;
use std::{collections::BinaryHeap, ops::Range, time::Instant};

/// Train a single tree.
#[allow(clippy::too_many_arguments)]
pub fn train(
	binned_features: ArrayView2<u8>,
	include_features: &[bool],
	gradients: &[f32],
	hessians: &[f32],
	ordered_gradients: &mut [f32],
	ordered_hessians: &mut [f32],
	examples_index: &mut [usize],
	examples_index_left: &mut [usize],
	examples_index_right: &mut [usize],
	bin_stats_pool: &mut BinStatsPool,
	hessians_are_constant: bool,
	options: &types::TrainOptions,
	timing: &timing::Timing,
) -> (TrainTree, Vec<(Range<usize>, f32)>) {
	// This is the tree returned by this function
	let mut tree = TrainTree { nodes: Vec::new() };
	// This priority queue stores the potential nodes to split ordered by their gain.
	let mut queue: BinaryHeap<QueueItem> = BinaryHeap::new();
	// To update the gradients and hessians we need to make predictions.
	// Rather than running each example through the tree, we can reuse
	// the mapping from example index to leaf value previously computed.
	let mut leaf_values: Vec<(Range<usize>, f32)> = Vec::new();

	// Compute the sums of gradients and hessians for the root node.
	let n_examples = gradients.len();
	let examples_index_range = 0..n_examples;
	let sum_gradients = gradients.into_par_iter().map(|v| v.to_f64().unwrap()).sum();
	let sum_hessians = if hessians_are_constant {
		n_examples.to_f64().unwrap()
	} else {
		hessians.into_par_iter().map(|v| v.to_f64().unwrap()).sum()
	};

	// If there are too few training examples or the hessians are too small,
	// just return a tree with a single leaf.
	if n_examples < 2 * options.min_examples_leaf
		|| sum_hessians < 2.0 * options.min_sum_hessians_in_leaf.to_f64().unwrap()
	{
		let value = compute_leaf_value(sum_gradients, sum_hessians, options);
		let node = TrainNode::Leaf(TrainLeafNode {
			value,
			examples_fraction: 1.0,
		});
		tree.nodes.push(node);
		leaf_values.push((examples_index_range, value));
		return (tree, leaf_values);
	}

	// compute the bin stats for the root node
	let start = Instant::now();
	let mut root_bin_stats = bin_stats_pool.get();
	compute_bin_stats_for_root_node(
		&mut root_bin_stats,
		&include_features,
		binned_features,
		gradients,
		hessians,
		hessians_are_constant,
	);
	timing.bin_stats.compute_bin_stats_root.inc(start.elapsed());

	// based on the node stats and bin stats, find a split, if any.
	let start = Instant::now();
	let find_split_output = find_split(
		&root_bin_stats,
		&include_features,
		sum_gradients,
		sum_hessians,
		examples_index_range.clone(),
		&options,
	);
	timing.find_split.inc(start.elapsed());

	// if we were able to find a split for the root node,
	// add it to the queue and proceed to the loop.
	// Otherwise, return a tree with a single node.
	if let Some(find_split_output) = find_split_output {
		queue.push(QueueItem {
			depth: 0,
			examples_index_range,
			gain: find_split_output.gain,
			left_n_examples: find_split_output.left_n_examples,
			left_sum_gradients: find_split_output.left_sum_gradients,
			left_sum_hessians: find_split_output.left_sum_hessians,
			bin_stats: root_bin_stats,
			parent_index: None,
			right_n_examples: find_split_output.right_n_examples,
			right_sum_gradients: find_split_output.right_sum_gradients,
			right_sum_hessians: find_split_output.right_sum_hessians,
			split_direction: None,
			split: find_split_output.split,
			sum_gradients,
			sum_hessians,
		});
	} else {
		let value = compute_leaf_value(sum_gradients, sum_hessians, options);
		let examples_count = examples_index_range.len();
		leaf_values.push((examples_index_range, value));
		let node = TrainNode::Leaf(TrainLeafNode {
			value,
			examples_fraction: examples_count.to_f32().unwrap() / n_examples.to_f32().unwrap(),
		});
		tree.nodes.push(node);
		// return the bin stats to the pool
		bin_stats_pool.items.push(root_bin_stats);
		return (tree, leaf_values);
	}

	while let Some(queue_item) = queue.pop() {
		// Update the node's parent left or right child index with the current node index
		// There are two cases:
		// 1. The current node's split direction is left: it is a left child of its parent
		// 2. The current node's split_direction is right: it is a right child of its parent
		let node_index = tree.nodes.len();
		if let Some(parent_index) = queue_item.parent_index {
			let parent = tree
				.nodes
				.get_mut(parent_index)
				.unwrap()
				.as_branch_mut()
				.unwrap();
			let split_direction = queue_item.split_direction.unwrap();
			match split_direction {
				types::SplitDirection::Left => parent.left_child_index = Some(node_index),
				types::SplitDirection::Right => parent.right_child_index = Some(node_index),
			}
		}

		// Determine the current number of leaf nodes if training were to stop now.
		// If the max leaf nodes is reached, add the current node as a leaf and continue
		// until all items are removed from the queue and added to the tree as leaves
		let n_leaf_nodes = leaf_values.len() + queue.len() + 1;
		let max_leaf_nodes_reached = n_leaf_nodes == options.max_leaf_nodes;
		if max_leaf_nodes_reached {
			let value =
				compute_leaf_value(queue_item.sum_gradients, queue_item.sum_hessians, options);
			let examples_count = queue_item.examples_index_range.len();
			let node = TrainNode::Leaf(TrainLeafNode {
				value,
				examples_fraction: examples_count.to_f32().unwrap() / n_examples.to_f32().unwrap(),
			});
			leaf_values.push((queue_item.examples_index_range.clone(), value));
			tree.nodes.push(node);
			bin_stats_pool.items.push(queue_item.bin_stats);
			continue;
		}

		// Add the current node to the tree
		// The missing values direction is the direction with more training examples
		// TODO: this is the naiive implementation that does not take into account
		// the split with the highest gain of missing values going left/right during training
		let missing_values_direction = if queue_item.left_n_examples > queue_item.right_n_examples {
			types::SplitDirection::Left
		} else {
			types::SplitDirection::Right
		};

		tree.nodes.push(TrainNode::Branch(TrainBranchNode {
			split: queue_item.split.clone(),
			left_child_index: None,
			right_child_index: None,
			missing_values_direction,
			examples_fraction: queue_item.examples_index_range.len().to_f32().unwrap()
				/ n_examples.to_f32().unwrap(),
		}));

		// rearrange the examples index
		let start_time = Instant::now();
		let (left, right) = rearrange_examples_index(
			binned_features,
			&queue_item.split,
			examples_index
				.get_mut(queue_item.examples_index_range.clone())
				.unwrap(),
			examples_index_left
				.get_mut(queue_item.examples_index_range.clone())
				.unwrap(),
			examples_index_right
				.get_mut(queue_item.examples_index_range.clone())
				.unwrap(),
		);
		// The left and right ranges are local to the node,
		// so add the node's start to make them global.
		let start = queue_item.examples_index_range.start;
		let left_examples_index_range = start + left.start..start + left.end;
		let right_examples_index_range = start + right.start..start + right.end;
		timing.rearrange_examples_index.inc(start_time.elapsed());

		// Determine if we should split left and/or right
		// based on the number of examples in the node
		// and the node's depth in the tree
		let max_depth_reached = queue_item.depth + 1 == options.max_depth;
		let should_split_left =
			!max_depth_reached && left_examples_index_range.len() >= options.min_examples_leaf * 2;
		let should_split_right =
			!max_depth_reached && right_examples_index_range.len() >= options.min_examples_leaf * 2;

		// If we should not split left, add a leaf.
		if !should_split_left {
			let left_child_index = tree.nodes.len();
			let value = compute_leaf_value(
				queue_item.left_sum_gradients,
				queue_item.left_sum_hessians,
				options,
			);
			let node = TrainNode::Leaf(TrainLeafNode {
				value,
				examples_fraction: queue_item.left_n_examples.to_f32().unwrap()
					/ n_examples.to_f32().unwrap(),
			});
			leaf_values.push((left_examples_index_range.clone(), value));
			tree.nodes.push(node);
			// set the parent's child index to the new node's index
			let parent = tree
				.nodes
				.get_mut(node_index)
				.unwrap()
				.as_branch_mut()
				.unwrap();
			parent.left_child_index = Some(left_child_index);
		}

		// If we should not split right, add a leaf.
		if !should_split_right {
			let right_child_index = tree.nodes.len();
			let value = compute_leaf_value(
				queue_item.right_sum_gradients,
				queue_item.right_sum_hessians,
				options,
			);
			let node = TrainNode::Leaf(TrainLeafNode {
				value,
				examples_fraction: queue_item.right_n_examples.to_f32().unwrap()
					/ n_examples.to_f32().unwrap(),
			});
			leaf_values.push((right_examples_index_range.clone(), value));
			tree.nodes.push(node);
			// set the parent's child index to the new node's index
			let parent = tree
				.nodes
				.get_mut(node_index)
				.unwrap()
				.as_branch_mut()
				.unwrap();
			parent.right_child_index = Some(right_child_index);
		}

		// If we should not split either left or right,
		// then there is nothing left to do, so we can
		// go to the next item on the queue.
		if !should_split_left && !should_split_right {
			// return the bin stats to the pool
			bin_stats_pool.items.push(queue_item.bin_stats);
			continue;
		}

		// Next, we compute the bin stats for the two children.
		// smaller_direction is the direction of the child with fewer examples.
		let smaller_direction =
			if left_examples_index_range.len() < right_examples_index_range.len() {
				types::SplitDirection::Left
			} else {
				types::SplitDirection::Right
			};
		let smaller_child_examples_index = match smaller_direction {
			types::SplitDirection::Left => &examples_index[left_examples_index_range.clone()],
			types::SplitDirection::Right => &examples_index[right_examples_index_range.clone()],
		};
		let mut smaller_child_bin_stats = bin_stats_pool.get();

		// Compute the bin stats for the child with fewer examples.
		let start = Instant::now();
		compute_bin_stats_for_non_root_node(
			&mut smaller_child_bin_stats,
			include_features,
			ordered_gradients,
			ordered_hessians,
			binned_features,
			gradients,
			hessians,
			hessians_are_constant,
			smaller_child_examples_index,
		);
		timing.bin_stats.compute_bin_stats.inc(start.elapsed());

		// Compute the bin stats for the child with more examples
		// by subtracting the bin stats of the child with fewer
		// examples from the parent's bin stats.
		let start = Instant::now();
		let mut larger_child_bin_stats = queue_item.bin_stats;
		compute_bin_stats_subtraction(&mut larger_child_bin_stats, &smaller_child_bin_stats);
		timing
			.bin_stats
			.compute_bin_stats_subtraction
			.inc(start.elapsed());
		let (left_bin_stats, right_bin_stats) = match smaller_direction {
			types::SplitDirection::Left => (smaller_child_bin_stats, larger_child_bin_stats),
			types::SplitDirection::Right => (larger_child_bin_stats, smaller_child_bin_stats),
		};

		// If both left and right should split, find the splits for both at the same
		// time. Allows for a slight speedup because of cache.
		let (left_find_split_output, right_find_split_output) =
			if should_split_left && should_split_right {
				// based on the node stats and bin stats, find a split, if any.
				let start = Instant::now();
				let (left_find_split_output, right_find_split_output) = find_split_both(
					&left_bin_stats,
					queue_item.left_sum_gradients,
					queue_item.left_sum_hessians,
					left_examples_index_range.clone(),
					&right_bin_stats,
					queue_item.right_sum_gradients,
					queue_item.right_sum_hessians,
					right_examples_index_range.clone(),
					include_features,
					&options,
				);
				timing.find_split.inc(start.elapsed());
				(left_find_split_output, right_find_split_output)
			} else if should_split_left {
				// based on the node stats and bin stats, find a split, if any.
				let start = Instant::now();
				let find_split_output = find_split(
					&left_bin_stats,
					&include_features,
					queue_item.left_sum_gradients,
					queue_item.left_sum_hessians,
					left_examples_index_range.clone(),
					&options,
				);
				timing.find_split.inc(start.elapsed());
				(find_split_output, None)
			} else if should_split_right {
				// based on the node stats and bin stats, find a split, if any.
				let start = Instant::now();
				let find_split_output = find_split(
					&right_bin_stats,
					&include_features,
					queue_item.right_sum_gradients,
					queue_item.right_sum_hessians,
					right_examples_index_range.clone(),
					&options,
				);
				timing.find_split.inc(start.elapsed());
				(None, find_split_output)
			} else {
				(None, None)
			};

		// if we were able to find a split for the node,
		// add it to the queue. Otherwise, add a leaf.
		if should_split_left {
			if let Some(find_split_output) = left_find_split_output {
				queue.push(QueueItem {
					depth: queue_item.depth + 1,
					examples_index_range: left_examples_index_range.clone(),
					gain: find_split_output.gain,
					left_n_examples: find_split_output.left_n_examples,
					left_sum_gradients: find_split_output.left_sum_gradients,
					left_sum_hessians: find_split_output.left_sum_hessians,
					bin_stats: left_bin_stats,
					parent_index: Some(node_index),
					right_n_examples: find_split_output.right_n_examples,
					right_sum_gradients: find_split_output.right_sum_gradients,
					right_sum_hessians: find_split_output.right_sum_hessians,
					split_direction: Some(types::SplitDirection::Left),
					split: find_split_output.split,
					sum_gradients: queue_item.left_sum_gradients,
					sum_hessians: queue_item.left_sum_hessians,
				});
			} else {
				let left_child_index = tree.nodes.len();
				let value = compute_leaf_value(sum_gradients, sum_hessians, options);
				leaf_values.push((left_examples_index_range, value));
				let node = TrainNode::Leaf(TrainLeafNode {
					value,
					examples_fraction: queue_item.left_n_examples.to_f32().unwrap()
						/ n_examples.to_f32().unwrap(),
				});
				tree.nodes.push(node);
				// set the parent's left child index to the new node's index
				let parent = tree
					.nodes
					.get_mut(node_index)
					.unwrap()
					.as_branch_mut()
					.unwrap();
				parent.left_child_index = Some(left_child_index);
				// return the bin stats to the pool
				bin_stats_pool.items.push(left_bin_stats);
			}
		} else {
			bin_stats_pool.items.push(left_bin_stats);
		}

		// If we were able to find a split for the node,
		// add it to the queue. Otherwise, add a leaf.
		if should_split_right {
			if let Some(find_split_output) = right_find_split_output {
				queue.push(QueueItem {
					depth: queue_item.depth + 1,
					examples_index_range: right_examples_index_range.clone(),
					gain: find_split_output.gain,
					left_n_examples: find_split_output.left_n_examples,
					left_sum_gradients: find_split_output.left_sum_gradients,
					left_sum_hessians: find_split_output.left_sum_hessians,
					bin_stats: right_bin_stats,
					parent_index: Some(node_index),
					right_n_examples: find_split_output.right_n_examples,
					right_sum_gradients: find_split_output.right_sum_gradients,
					right_sum_hessians: find_split_output.right_sum_hessians,
					split_direction: Some(types::SplitDirection::Right),
					split: find_split_output.split,
					sum_gradients: queue_item.right_sum_gradients,
					sum_hessians: queue_item.right_sum_hessians,
				});
			} else {
				let right_child_index = tree.nodes.len();
				let value = compute_leaf_value(sum_gradients, sum_hessians, options);
				leaf_values.push((right_examples_index_range, value));
				let node = TrainNode::Leaf(TrainLeafNode {
					value,
					examples_fraction: queue_item.right_n_examples.to_f32().unwrap()
						/ n_examples.to_f32().unwrap(),
				});
				tree.nodes.push(node);
				// set the parent's left child index to the new node's index
				let parent = tree
					.nodes
					.get_mut(node_index)
					.unwrap()
					.as_branch_mut()
					.unwrap();
				parent.right_child_index = Some(right_child_index);
				// return the bin stats to the pool
				bin_stats_pool.items.push(right_bin_stats);
			}
		} else {
			bin_stats_pool.items.push(right_bin_stats)
		}
	}

	(TrainTree { nodes: tree.nodes }, leaf_values)
}

/// Compute the value for a leaf node.
#[inline(always)]
fn compute_leaf_value(sum_gradients: f64, sum_hessians: f64, options: &types::TrainOptions) -> f32 {
	(-options.learning_rate.to_f64().unwrap() * sum_gradients
		/ (sum_hessians + options.l2_regularization.to_f64().unwrap() + std::f64::EPSILON))
		.to_f32()
		.unwrap()
}
