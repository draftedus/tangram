use super::{
	bin::BinningInstructions,
	bin_stats::{BinStats, BinStatsEntry},
	TrainBranchSplit, TrainBranchSplitContinuous, TrainBranchSplitDiscrete,
};
use crate::{SplitDirection, TrainOptions};
use num_traits::ToPrimitive;
use rayon::prelude::*;
use std::ops::Range;

pub struct ChooseBestSplitOutput {
	pub gain: f32,
	pub split: TrainBranchSplit,
	pub left_n_examples: usize,
	pub left_sum_gradients: f64,
	pub left_sum_hessians: f64,
	pub right_n_examples: usize,
	pub right_sum_gradients: f64,
	pub right_sum_hessians: f64,
}

/// Choose the split with the highest gain, if a valid one exists.
pub fn choose_best_split(
	bin_stats: &BinStats,
	sum_gradients: f64,
	sum_hessians: f64,
	examples_index_range: Range<usize>,
	options: &TrainOptions,
) -> Option<ChooseBestSplitOutput> {
	(&bin_stats.entries, &bin_stats.binning_instructions)
		.into_par_iter()
		.enumerate()
		.filter_map(
			|(feature_index, (bin_stats, binning_instructions))| match binning_instructions {
				BinningInstructions::Number { .. } => choose_best_split_continuous(
					feature_index,
					&binning_instructions,
					bin_stats,
					sum_gradients,
					sum_hessians,
					examples_index_range.clone(),
					options,
				),
				BinningInstructions::Enum { .. } => choose_best_split_discrete(
					feature_index,
					&binning_instructions,
					bin_stats,
					sum_gradients,
					sum_hessians,
					examples_index_range.clone(),
					options,
				),
			},
		)
		.max_by(|a, b| a.gain.partial_cmp(&b.gain).unwrap())
}

/// Choose the best continuous split for this feature.
fn choose_best_split_continuous(
	feature_index: usize,
	binning_instructions: &BinningInstructions,
	bin_stats_for_feature: &[BinStatsEntry],
	sum_gradients_parent: f64,
	sum_hessians_parent: f64,
	examples_index_range: Range<usize>,
	options: &TrainOptions,
) -> Option<ChooseBestSplitOutput> {
	let mut best_split: Option<ChooseBestSplitOutput> = None;
	let negative_loss_parent_node = compute_negative_loss(
		sum_gradients_parent,
		sum_hessians_parent,
		options.l2_regularization,
	);
	let count_multiplier = examples_index_range.len() as f64 / sum_hessians_parent;
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
			let right_n_examples = match examples_index_range.len().checked_sub(left_n_examples) {
				Some(right_n_examples) => right_n_examples,
				None => break,
			};
			let right_sum_gradients = sum_gradients_parent - left_sum_gradients;
			let right_sum_hessians = sum_hessians_parent - left_sum_hessians;
			// check if we have violated the min samples leaf constraint
			if left_n_examples < options.min_examples_per_child {
				continue;
			}
			// Since we are in left to right mode, we will only get less examples if we continue so break instead.
			if right_n_examples < options.min_examples_per_child {
				break;
			}
			// If hessians are positive so the left sum hessians will continue to increase, so we can continue.
			if left_sum_hessians < options.min_sum_hessians_per_child.to_f64().unwrap() {
				continue;
			}
			// If hessians are positive so we will continue to violate the min_hessian_to_split condition for the right node, break.
			if right_sum_hessians < options.min_sum_hessians_per_child.to_f64().unwrap() {
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
						Some(i) => thresholds.get(i).unwrap(),
						None => f32::MIN,
					},
					_ => unreachable!(),
				},
				invalid_values_direction,
			});
			let current_split = ChooseBestSplitOutput {
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
fn choose_best_split_discrete(
	feature_index: usize,
	binning_instructions: &BinningInstructions,
	bin_stats_for_feature: &[BinStatsEntry],
	sum_gradients_parent: f64,
	sum_hessians_parent: f64,
	examples_index_range: Range<usize>,
	options: &TrainOptions,
) -> Option<ChooseBestSplitOutput> {
	let mut best_split_so_far: Option<ChooseBestSplitOutput> = None;
	let training_data_contains_invalid_values = bin_stats_for_feature[0].sum_hessians > 0.0;
	let l2_regularization =
		options.l2_regularization + options.supplemental_l2_regularization_for_discrete_splits;
	let negative_loss_parent_node =
		compute_negative_loss(sum_gradients_parent, sum_hessians_parent, l2_regularization);
	let count_multiplier = examples_index_range.len() as f64 / sum_hessians_parent;
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
		let right_n_examples = match examples_index_range.len().checked_sub(left_n_examples) {
			Some(right_n_examples) => right_n_examples,
			None => break,
		};
		// check if we have violated the min samples leaf constraint
		if left_n_examples < options.min_examples_per_child {
			continue;
		}
		if right_n_examples < options.min_examples_per_child {
			// since we are in left to right mode, we will only get less examples if we continue so break instead
			break;
		}
		if left_sum_hessians < options.min_sum_hessians_per_child.to_f64().unwrap() {
			// Hessians are positive so the left sum hessians will continue to increase,
			// we can continue.
			continue;
		}
		if right_sum_hessians < options.min_sum_hessians_per_child.to_f64().unwrap() {
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
		let current_split = ChooseBestSplitOutput {
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
