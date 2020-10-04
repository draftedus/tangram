use super::{
	bin_stats::{
		compute_bin_stats_for_feature_not_root, compute_bin_stats_for_feature_not_root_subtraction,
		compute_bin_stats_for_feature_root, BinStats, BinStatsEntry,
	},
	binning::{BinnedFeatures, BinningInstructions},
	TrainBranchSplit, TrainBranchSplitContinuous, TrainBranchSplitDiscrete,
};
#[cfg(feature = "timing")]
use crate::timing::Timing;
use crate::{SplitDirection, TrainOptions};
use itertools::izip;
use num_traits::ToPrimitive;
use rayon::prelude::*;
use tangram_pool::{Pool, PoolGuard};
use tangram_thread_pool::pzip;

pub enum ChooseBestSplitOutput {
	Success(ChooseBestSplitSuccess),
	Failure(ChooseBestSplitFailure),
}

pub struct ChooseBestSplitSuccess {
	pub gain: f32,
	pub split: TrainBranchSplit,
	pub sum_gradients: f64,
	pub sum_hessians: f64,
	pub left_n_examples: usize,
	pub left_sum_gradients: f64,
	pub left_sum_hessians: f64,
	pub right_n_examples: usize,
	pub right_sum_gradients: f64,
	pub right_sum_hessians: f64,
	pub bin_stats: PoolGuard<BinStats>,
}

pub struct ChooseBestSplitFailure {
	pub sum_gradients: f64,
	pub sum_hessians: f64,
}

pub struct ChooseBestSplitForFeatureOutput {
	pub gain: f32,
	pub split: TrainBranchSplit,
	pub left_n_examples: usize,
	pub left_sum_gradients: f64,
	pub left_sum_hessians: f64,
	pub right_n_examples: usize,
	pub right_sum_gradients: f64,
	pub right_sum_hessians: f64,
}

#[allow(clippy::too_many_arguments)]
pub fn choose_best_split_root(
	bin_stats_pool: &mut Pool<BinStats>,
	binning_instructions: &[BinningInstructions],
	binned_features: &BinnedFeatures,
	gradients: &[f32],
	hessians: &[f32],
	hessians_are_constant: bool,
	options: &TrainOptions,
	#[cfg(feature = "timing")] timing: &Timing,
) -> ChooseBestSplitOutput {
	// Compute the sums of gradients and hessians.
	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	let sum_gradients = gradients
		.par_iter()
		.map(|gradient| gradient.to_f64().unwrap())
		.sum::<f64>();
	let sum_hessians = if hessians_are_constant {
		hessians.len().to_f64().unwrap()
	} else {
		hessians
			.par_iter()
			.map(|hessian| hessian.to_f64().unwrap())
			.sum::<f64>()
	};
	#[cfg(feature = "timing")]
	timing.sum_gradients_and_hessians_root.inc(start.elapsed());

	// Determine if we should try to split the root.
	let should_try_to_split_root = gradients.len() >= 2 * options.min_examples_per_node
		&& sum_hessians >= 2.0 * options.min_sum_hessians_per_node.to_f64().unwrap();
	if !should_try_to_split_root {
		return ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
			sum_gradients,
			sum_hessians,
		});
	}

	// For each feature, compute bin stats and use them to choose the best split.
	let mut bin_stats = bin_stats_pool.get().unwrap();
	let best_split_output: Option<ChooseBestSplitForFeatureOutput> = pzip!(
		binning_instructions,
		&binned_features.columns,
		&mut bin_stats.0
	)
	.enumerate()
	.map(
		|(feature_index, (binning_instructions, binned_feature, bin_stats_for_feature))| {
			// Compute the bin stats.
			compute_bin_stats_for_feature_root(
				bin_stats_for_feature,
				binned_feature,
				gradients,
				hessians,
				hessians_are_constant,
			);
			// Choose the best split for this featue.
			choose_best_split_for_feature(
				feature_index,
				binning_instructions,
				bin_stats_for_feature,
				binned_feature.len(),
				sum_gradients,
				sum_hessians,
				options,
			)
		},
	)
	.filter_map(|split| split)
	.max_by(|a, b| a.gain.partial_cmp(&b.gain).unwrap());

	// Assemble the output.
	match best_split_output {
		Some(best_split) => ChooseBestSplitOutput::Success(ChooseBestSplitSuccess {
			gain: best_split.gain,
			split: best_split.split,
			sum_gradients,
			sum_hessians,
			left_n_examples: best_split.left_n_examples,
			left_sum_gradients: best_split.left_sum_gradients,
			left_sum_hessians: best_split.left_sum_hessians,
			right_n_examples: best_split.right_n_examples,
			right_sum_gradients: best_split.right_sum_gradients,
			right_sum_hessians: best_split.right_sum_hessians,
			bin_stats,
		}),
		None => ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
			sum_gradients,
			sum_hessians,
		}),
	}
}

#[allow(clippy::too_many_arguments)]
pub fn choose_best_splits_not_root(
	bin_stats_pool: &mut Pool<BinStats>,
	binning_instructions: &[BinningInstructions],
	binned_features: &BinnedFeatures,
	parent_depth: usize,
	gradients: &[f32],
	hessians: &[f32],
	left_child_n_examples: usize,
	left_child_sum_gradients: f64,
	left_child_sum_hessians: f64,
	right_child_n_examples: usize,
	right_child_sum_gradients: f64,
	right_child_sum_hessians: f64,
	left_child_examples_index: &[i32],
	right_child_examples_index: &[i32],
	gradients_ordered_buffer: &mut [f32],
	hessians_ordered_buffer: &mut [f32],
	parent_bin_stats: PoolGuard<BinStats>,
	hessians_are_constant: bool,
	options: &TrainOptions,
	#[cfg(feature = "timing")] timing: &Timing,
) -> (ChooseBestSplitOutput, ChooseBestSplitOutput) {
	let mut left_child_output = ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
		sum_gradients: left_child_sum_gradients,
		sum_hessians: left_child_sum_hessians,
	});
	let mut right_child_output = ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
		sum_gradients: right_child_sum_gradients,
		sum_hessians: right_child_sum_hessians,
	});

	// Determine if we should try to split the left and/or right children of this branch.
	let children_will_exceed_max_depth = if let Some(max_depth) = options.max_depth {
		parent_depth + 1 > max_depth - 1
	} else {
		false
	};
	let should_try_to_split_left_child = !children_will_exceed_max_depth
		&& left_child_examples_index.len() >= options.min_examples_per_node * 2;
	let should_try_to_split_right_child = !children_will_exceed_max_depth
		&& right_child_examples_index.len() >= options.min_examples_per_node * 2;

	// If we should not split either left or right, then there is nothing left to do, so we can go to the next item on the queue.
	if !should_try_to_split_left_child && !should_try_to_split_right_child {
		return (left_child_output, right_child_output);
	}

	// Determine which of the left and right children have fewer examples sent to them.
	let smaller_child_direction =
		if left_child_examples_index.len() < right_child_examples_index.len() {
			SplitDirection::Left
		} else {
			SplitDirection::Right
		};
	let smaller_child_examples_index = match smaller_child_direction {
		SplitDirection::Left => left_child_examples_index,
		SplitDirection::Right => right_child_examples_index,
	};
	let mut smaller_child_bin_stats = bin_stats_pool.get().unwrap();
	let mut larger_child_bin_stats = parent_bin_stats;

	// Fill the gradients and hessians ordered buffers. The buffers contain the gradients and hessians for each example as ordered by the examples index. This makes the access of the gradients and hessians sequential in the next step.
	fill_gradients_and_hessians_ordered_buffers(
		smaller_child_examples_index,
		gradients,
		hessians,
		gradients_ordered_buffer,
		hessians_ordered_buffer,
		hessians_are_constant,
	);

	let children_best_splits: Vec<(
		Option<ChooseBestSplitForFeatureOutput>,
		Option<ChooseBestSplitForFeatureOutput>,
	)> = pzip!(
		binning_instructions,
		&binned_features.columns,
		&mut smaller_child_bin_stats.0,
		&mut larger_child_bin_stats.0,
	)
	.enumerate()
	.map(
		|(
			feature_index,
			(
				binning_instructions,
				binned_features_column,
				smaller_child_bin_stats_for_feature,
				mut larger_child_bin_stats_for_feature,
			),
		)| {
			// Compute the bin stats for the child with fewer examples.
			#[cfg(feature = "timing")]
			let start = std::time::Instant::now();
			compute_bin_stats_for_feature_not_root(
				smaller_child_bin_stats_for_feature,
				smaller_child_examples_index,
				binned_features_column,
				gradients_ordered_buffer,
				hessians_ordered_buffer,
				hessians_are_constant,
			);
			#[cfg(feature = "timing")]
			timing.compute_bin_stats_not_root.inc(start.elapsed());

			// Compute the larger child bin stats by subtraction.
			compute_bin_stats_for_feature_not_root_subtraction(
				&smaller_child_bin_stats_for_feature,
				&mut larger_child_bin_stats_for_feature,
			);

			// Assign the smaller and larger bin stats to the left and right children depending on which direction was smaller.
			let (left_child_bin_stats_for_feature, right_child_bin_stats_for_feature) =
				match smaller_child_direction {
					SplitDirection::Left => (
						smaller_child_bin_stats_for_feature,
						larger_child_bin_stats_for_feature,
					),
					SplitDirection::Right => (
						larger_child_bin_stats_for_feature,
						smaller_child_bin_stats_for_feature,
					),
				};

			// Choose the best splits for the left and right children.
			let left_child_best_split_for_feature = if should_try_to_split_left_child {
				choose_best_split_for_feature(
					feature_index,
					binning_instructions,
					left_child_bin_stats_for_feature,
					left_child_n_examples,
					left_child_sum_gradients,
					left_child_sum_hessians,
					options,
				)
			} else {
				None
			};
			let right_child_best_split_for_feature = if should_try_to_split_right_child {
				choose_best_split_for_feature(
					feature_index,
					binning_instructions,
					right_child_bin_stats_for_feature,
					right_child_n_examples,
					right_child_sum_gradients,
					right_child_sum_hessians,
					options,
				)
			} else {
				None
			};

			(
				left_child_best_split_for_feature,
				right_child_best_split_for_feature,
			)
		},
	)
	.collect();
	let (left_child_best_split, right_child_best_split) = children_best_splits.into_iter().fold(
		(None, None),
		|(current_left, current_right), (candidate_left, candidate_right)| {
			(
				match (current_left, candidate_left) {
					(None, None) => None,
					(x, None) => x,
					(None, x) => x,
					(Some(current), Some(candidate)) => {
						if candidate.gain > current.gain {
							Some(candidate)
						} else {
							Some(current)
						}
					}
				},
				match (current_right, candidate_right) {
					(None, None) => None,
					(x, None) => x,
					(None, x) => x,
					(Some(current), Some(candidate)) => {
						if candidate.gain > current.gain {
							Some(candidate)
						} else {
							Some(current)
						}
					}
				},
			)
		},
	);

	// Assign the smaller and larger bin stats to the left and right children depending on which direction was smaller.
	let (left_child_bin_stats, right_child_bin_stats) = match smaller_child_direction {
		SplitDirection::Left => (smaller_child_bin_stats, larger_child_bin_stats),
		SplitDirection::Right => (larger_child_bin_stats, smaller_child_bin_stats),
	};

	// Assemble the output.
	left_child_output = match left_child_best_split {
		Some(best_split) => ChooseBestSplitOutput::Success(ChooseBestSplitSuccess {
			gain: best_split.gain,
			split: best_split.split,
			sum_gradients: left_child_sum_gradients,
			sum_hessians: left_child_sum_hessians,
			left_n_examples: best_split.left_n_examples,
			left_sum_gradients: best_split.left_sum_gradients,
			left_sum_hessians: best_split.left_sum_hessians,
			right_n_examples: best_split.right_n_examples,
			right_sum_gradients: best_split.right_sum_gradients,
			right_sum_hessians: best_split.right_sum_hessians,
			bin_stats: left_child_bin_stats,
		}),
		None => ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
			sum_gradients: left_child_sum_gradients,
			sum_hessians: left_child_sum_hessians,
		}),
	};
	right_child_output = match right_child_best_split {
		Some(best_split) => ChooseBestSplitOutput::Success(ChooseBestSplitSuccess {
			gain: best_split.gain,
			split: best_split.split,
			sum_gradients: right_child_sum_gradients,
			sum_hessians: right_child_sum_hessians,
			left_n_examples: best_split.left_n_examples,
			left_sum_gradients: best_split.left_sum_gradients,
			left_sum_hessians: best_split.left_sum_hessians,
			right_n_examples: best_split.right_n_examples,
			right_sum_gradients: best_split.right_sum_gradients,
			right_sum_hessians: best_split.right_sum_hessians,
			bin_stats: right_child_bin_stats,
		}),
		None => ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
			sum_gradients: right_child_sum_gradients,
			sum_hessians: right_child_sum_hessians,
		}),
	};
	(left_child_output, right_child_output)
}

fn choose_best_split_for_feature(
	feature_index: usize,
	binning_instructions: &BinningInstructions,
	bin_stats_for_feature: &[BinStatsEntry],
	n_examples: usize,
	sum_gradients: f64,
	sum_hessians: f64,
	options: &TrainOptions,
) -> Option<ChooseBestSplitForFeatureOutput> {
	match binning_instructions {
		BinningInstructions::Number { .. } => choose_best_split_for_continuous_feature(
			feature_index,
			&binning_instructions,
			bin_stats_for_feature,
			n_examples,
			sum_gradients,
			sum_hessians,
			options,
		),
		BinningInstructions::Enum { .. } => choose_best_split_for_discrete_feature(
			feature_index,
			&binning_instructions,
			bin_stats_for_feature,
			n_examples,
			sum_gradients,
			sum_hessians,
			options,
		),
	}
}

/// Choose the best continuous split for this feature.
fn choose_best_split_for_continuous_feature(
	feature_index: usize,
	binning_instructions: &BinningInstructions,
	bin_stats_for_feature: &[BinStatsEntry],
	n_examples_parent: usize,
	sum_gradients_parent: f64,
	sum_hessians_parent: f64,
	options: &TrainOptions,
) -> Option<ChooseBestSplitForFeatureOutput> {
	let mut best_split: Option<ChooseBestSplitForFeatureOutput> = None;
	let negative_loss_parent_node = compute_negative_loss(
		sum_gradients_parent,
		sum_hessians_parent,
		options.l2_regularization,
	);
	let count_multiplier = n_examples_parent.to_f64().unwrap() / sum_hessians_parent;
	let training_data_contains_invalid_values = bin_stats_for_feature[0].sum_hessians > 0.0;
	let choose_best_direction_for_invalid_values = training_data_contains_invalid_values;
	let invalid_values_directions = if choose_best_direction_for_invalid_values {
		vec![SplitDirection::Left, SplitDirection::Right]
	} else {
		vec![SplitDirection::Left]
	};
	for invalid_direction in invalid_values_directions {
		let mut left_sum_gradients = 0.0;
		let mut left_sum_hessians = 0.0;
		let mut left_n_examples = 0;
		let bin_stats_for_feature =
			match (choose_best_direction_for_invalid_values, invalid_direction) {
				(true, SplitDirection::Left) => {
					&bin_stats_for_feature[0..bin_stats_for_feature.len() - 1]
				}
				(true, SplitDirection::Right) => {
					&bin_stats_for_feature[1..bin_stats_for_feature.len()]
				}
				(false, _) => &bin_stats_for_feature[0..bin_stats_for_feature.len() - 1],
			};
		for (bin_index, bin_stats_entry) in bin_stats_for_feature.iter().enumerate() {
			// Approximate the number of examples that go left by assuming it is proporational to the sum of the hessians.
			left_n_examples += (bin_stats_entry.sum_hessians * count_multiplier)
				.round()
				.to_usize()
				.unwrap();
			left_sum_gradients += bin_stats_entry.sum_gradients;
			left_sum_hessians += bin_stats_entry.sum_hessians;
			let right_n_examples = match n_examples_parent.checked_sub(left_n_examples) {
				Some(right_n_examples) => right_n_examples,
				None => break,
			};
			let right_sum_gradients = sum_gradients_parent - left_sum_gradients;
			let right_sum_hessians = sum_hessians_parent - left_sum_hessians;
			// check if we have violated the min samples leaf constraint
			if left_n_examples < options.min_examples_per_node {
				continue;
			}
			// Since we are in left to right mode, we will only get less examples if we continue so break instead.
			if right_n_examples < options.min_examples_per_node {
				break;
			}
			// If hessians are positive so the left sum hessians will continue to increase, so we can continue.
			if left_sum_hessians < options.min_sum_hessians_per_node.to_f64().unwrap() {
				continue;
			}
			// If hessians are positive so we will continue to violate the min_hessian_to_split condition for the right node, break.
			if right_sum_hessians < options.min_sum_hessians_per_node.to_f64().unwrap() {
				break;
			}
			let current_split_gain = compute_gain(
				left_sum_gradients,
				left_sum_hessians,
				right_sum_gradients,
				right_sum_hessians,
				negative_loss_parent_node,
				options.l2_regularization,
			);
			// This is the direction invalid values should be sent.
			let invalid_values_direction = if choose_best_direction_for_invalid_values {
				invalid_direction
			} else {
				// Send invalid values to the child node where more training examples are sent.
				if left_n_examples >= right_n_examples {
					SplitDirection::Left
				} else {
					SplitDirection::Right
				}
			};
			let split = TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
				feature_index,
				bin_index,
				split_value: match binning_instructions {
					BinningInstructions::Number { thresholds } => match bin_index.checked_sub(1) {
						Some(i) => *thresholds.get(i).unwrap(),
						None => f32::MIN,
					},
					_ => unreachable!(),
				},
				invalid_values_direction,
			});
			let current_split = ChooseBestSplitForFeatureOutput {
				gain: current_split_gain,
				split,
				left_n_examples,
				left_sum_gradients,
				left_sum_hessians,
				right_n_examples,
				right_sum_gradients,
				right_sum_hessians,
			};
			match &best_split {
				Some(current_best_split) => {
					if current_split.gain > current_best_split.gain {
						best_split = Some(current_split);
					}
				}
				None => {
					if current_split.gain > options.min_gain_to_split {
						best_split = Some(current_split);
					}
				}
			}
		}
	}
	best_split
}

/**
Find the best split for this discrete (categorical) feature. A discrete split is a partition of the categories into two subsets where one subset goes to the left subtree and one goes to the right.
To find the subsets:
1. Sort the bins by sum_gradients / (sum_hessians + categorical_smoothing_factor).
2. Perform the same algorithm to find the best split as the continuous setting, but iterate bins in the sorted order defined in step 1.
*/
fn choose_best_split_for_discrete_feature(
	feature_index: usize,
	binning_instructions: &BinningInstructions,
	bin_stats_for_feature: &[BinStatsEntry],
	n_examples_parent: usize,
	sum_gradients_parent: f64,
	sum_hessians_parent: f64,
	options: &TrainOptions,
) -> Option<ChooseBestSplitForFeatureOutput> {
	let mut best_split_so_far: Option<ChooseBestSplitForFeatureOutput> = None;
	let training_data_contains_invalid_values = bin_stats_for_feature[0].sum_hessians > 0.0;
	let l2_regularization =
		options.l2_regularization + options.supplemental_l2_regularization_for_discrete_splits;
	let negative_loss_parent_node =
		compute_negative_loss(sum_gradients_parent, sum_hessians_parent, l2_regularization);
	let count_multiplier = n_examples_parent.to_f64().unwrap() / sum_hessians_parent;
	let mut left_sum_gradients = 0.0;
	let mut left_sum_hessians = 0.0;
	let mut left_n_examples = 0;
	let smoothing_factor = options
		.smoothing_factor_for_discrete_bin_sorting
		.to_f64()
		.unwrap();
	let categorical_bin_score =
		|bin: &BinStatsEntry| bin.sum_gradients / (bin.sum_hessians + smoothing_factor);
	let mut sorted_bin_stats: Vec<(usize, &BinStatsEntry)> =
		bin_stats_for_feature.iter().enumerate().collect();
	sorted_bin_stats.sort_by(|(_, a), (_, b)| {
		categorical_bin_score(a)
			.partial_cmp(&categorical_bin_score(b))
			.unwrap()
	});
	let mut directions = vec![SplitDirection::Right; binning_instructions.n_bins()];
	let iter = sorted_bin_stats[0..sorted_bin_stats.len() - 1].iter();
	for (bin_index, bin_stats_entry) in iter {
		*directions.get_mut(*bin_index).unwrap() = SplitDirection::Left;
		// Approximate the number of examples that go left by assuming it is proporational to the sum of the hessians.
		left_n_examples += (bin_stats_entry.sum_hessians * count_multiplier)
			.round()
			.to_usize()
			.unwrap();
		left_sum_gradients += bin_stats_entry.sum_gradients;
		left_sum_hessians += bin_stats_entry.sum_hessians;
		let right_sum_gradients = sum_gradients_parent - left_sum_gradients;
		let right_sum_hessians = sum_hessians_parent - left_sum_hessians;
		let right_n_examples = match n_examples_parent.checked_sub(left_n_examples) {
			Some(right_n_examples) => right_n_examples,
			None => break,
		};
		// check if we have violated the min samples leaf constraint
		if left_n_examples < options.min_examples_per_node {
			continue;
		}
		if right_n_examples < options.min_examples_per_node {
			// since we are in left to right mode, we will only get less examples if we continue so break instead
			break;
		}
		if left_sum_hessians < options.min_sum_hessians_per_node.to_f64().unwrap() {
			// Hessians are positive so the left sum hessians will continue to increase,
			// we can continue.
			continue;
		}
		if right_sum_hessians < options.min_sum_hessians_per_node.to_f64().unwrap() {
			// Hessians are positive so we will continue to violate the min_hessian_to_split
			// condition for the right node, break.
			break;
		}
		let current_split_gain = compute_gain(
			left_sum_gradients,
			left_sum_hessians,
			right_sum_gradients,
			right_sum_hessians,
			negative_loss_parent_node,
			l2_regularization,
		);
		if !training_data_contains_invalid_values {
			// no invalid values, send invalid values to the child with more examples
			if left_n_examples > right_n_examples {
				*directions.get_mut(0).unwrap() = SplitDirection::Left;
			} else {
				*directions.get_mut(0).unwrap() = SplitDirection::Right;
			}
		}
		let split = TrainBranchSplit::Discrete(TrainBranchSplitDiscrete {
			feature_index,
			directions: directions.clone(),
		});
		let current_split = ChooseBestSplitForFeatureOutput {
			gain: current_split_gain,
			left_n_examples,
			left_sum_gradients,
			left_sum_hessians,
			right_n_examples,
			right_sum_gradients,
			right_sum_hessians,
			split,
		};
		match &best_split_so_far {
			Some(current_best_split) => {
				if current_split.gain > current_best_split.gain {
					best_split_so_far = Some(current_split)
				}
			}
			None => {
				if current_split.gain > options.min_gain_to_split {
					best_split_so_far = Some(current_split)
				}
			}
		}
	}
	best_split_so_far
}

/// The gain is a value that is used to measure how good a given split is.
fn compute_gain(
	sum_gradients_left: f64,
	sum_hessians_left: f64,
	sum_gradients_right: f64,
	sum_hessians_right: f64,
	negative_loss_current_node: f32,
	l2_regularization: f32,
) -> f32 {
	let left = compute_negative_loss(sum_gradients_left, sum_hessians_left, l2_regularization);
	let right = compute_negative_loss(sum_gradients_right, sum_hessians_right, l2_regularization);
	left + right - negative_loss_current_node
}

/// The negative loss is used to compute the gain of a given split.
fn compute_negative_loss(sum_gradients: f64, sum_hessians: f64, l2_regularization: f32) -> f32 {
	((sum_gradients * sum_gradients) / (sum_hessians + l2_regularization.to_f64().unwrap()))
		.to_f32()
		.unwrap()
}

fn fill_gradients_and_hessians_ordered_buffers(
	smaller_child_examples_index: &[i32],
	gradients: &[f32],
	hessians: &[f32],
	gradients_ordered_buffer: &mut [f32],
	hessians_ordered_buffer: &mut [f32],
	hessians_are_constant: bool,
) {
	#[allow(clippy::collapsible_if)]
	if !hessians_are_constant {
		if smaller_child_examples_index.len() < 1024 {
			izip!(
				smaller_child_examples_index,
				&mut *gradients_ordered_buffer,
				&mut *hessians_ordered_buffer,
			)
			.for_each(
				|(example_index, ordered_gradient, ordered_hessian)| unsafe {
					*ordered_gradient = *gradients.get_unchecked(example_index.to_usize().unwrap());
					*ordered_hessian = *hessians.get_unchecked(example_index.to_usize().unwrap());
				},
			);
		} else {
			let chunk_size = smaller_child_examples_index.len() / rayon::current_num_threads();
			pzip!(
				smaller_child_examples_index.par_chunks(chunk_size),
				gradients_ordered_buffer.par_chunks_mut(chunk_size),
				hessians_ordered_buffer.par_chunks_mut(chunk_size),
			)
			.for_each(
				|(example_index_for_node, ordered_gradients, ordered_hessians)| {
					izip!(example_index_for_node, ordered_gradients, ordered_hessians).for_each(
						|(example_index, ordered_gradient, ordered_hessian)| unsafe {
							*ordered_gradient =
								*gradients.get_unchecked(example_index.to_usize().unwrap());
							*ordered_hessian =
								*hessians.get_unchecked(example_index.to_usize().unwrap());
						},
					);
				},
			);
		}
	} else {
		if smaller_child_examples_index.len() < 1024 {
			izip!(smaller_child_examples_index, &mut *gradients_ordered_buffer,).for_each(
				|(example_index, ordered_gradient)| unsafe {
					*ordered_gradient = *gradients.get_unchecked(example_index.to_usize().unwrap());
				},
			);
		} else {
			let chunk_size = smaller_child_examples_index.len() / rayon::current_num_threads();
			pzip!(
				smaller_child_examples_index.par_chunks(chunk_size),
				gradients_ordered_buffer.par_chunks_mut(chunk_size),
			)
			.for_each(|(example_index_for_node, ordered_gradients)| unsafe {
				izip!(example_index_for_node, ordered_gradients,).for_each(
					|(example_index, ordered_gradient)| {
						*ordered_gradient =
							*gradients.get_unchecked(example_index.to_usize().unwrap());
					},
				);
			});
		}
	}
}
