use super::{
	bin::BinningInstructions, bin_stats::BinStats, TrainBranchSplit, TrainBranchSplitContinuous,
	TrainBranchSplitDiscrete,
};
use crate::{SplitDirection, TrainOptions};
use itertools::izip;
use num_traits::ToPrimitive;
use std::ops::Range;

pub struct ChooseBestSplitOutput {
	pub gain: f32,
	pub feature_index: usize,
	pub split: TrainBranchSplit,
	pub left_sum_gradients: f64,
	pub left_sum_hessians: f64,
	pub left_n_examples: usize,
	pub right_sum_gradients: f64,
	pub right_sum_hessians: f64,
	pub right_n_examples: usize,
}

/// Find the split with the highest gain across all features, if a valid one exists.
pub fn choose_best_split(
	bin_stats: &BinStats,
	sum_gradients: f64,
	sum_hessians: f64,
	examples_index_range: Range<usize>,
	options: &TrainOptions,
) -> Option<ChooseBestSplitOutput> {
	izip!(&bin_stats.entries, &bin_stats.binning_instructions)
		.enumerate()
		.filter_map(
			|(feature_index, (bin_stats, binning_instructions))| match binning_instructions {
				BinningInstructions::Number { .. } => {
					find_best_continuous_split_for_feature_left_to_right(
						feature_index,
						&binning_instructions,
						bin_stats,
						sum_gradients,
						sum_hessians,
						examples_index_range.clone(),
						options,
					)
				}
				BinningInstructions::Enum { .. } => {
					find_best_discrete_split_for_feature_left_to_right(
						feature_index,
						&binning_instructions,
						bin_stats,
						sum_gradients,
						sum_hessians,
						examples_index_range.clone(),
						options,
					)
				}
			},
		)
		.max_by(|a, b| a.gain.partial_cmp(&b.gain).unwrap())
}

/// Find the split with the highest gain across all features for both of the left and right child at the same time, if a valid one exists. A valid split will not exist if the split conditions are violated for all potential splits. By looping over the features once, we increase the cache efficiency.
#[allow(clippy::too_many_arguments)]
pub fn choose_best_split_both(
	left_bin_stats: &BinStats,
	left_sum_gradients: f64,
	left_sum_hessians: f64,
	left_examples_index_range: Range<usize>,
	right_bin_stats: &BinStats,
	right_sum_gradients: f64,
	right_sum_hessians: f64,
	right_examples_index_range: Range<usize>,
	options: &TrainOptions,
) -> (Option<ChooseBestSplitOutput>, Option<ChooseBestSplitOutput>) {
	let best: Vec<(Option<ChooseBestSplitOutput>, Option<ChooseBestSplitOutput>)> = (0
		..left_bin_stats.entries.len())
		.map(|feature_index| {
			let binning_instructions = &left_bin_stats.binning_instructions[feature_index];
			match binning_instructions {
				BinningInstructions::Number { .. } => (
					find_best_continuous_split_for_feature_left_to_right(
						feature_index,
						binning_instructions,
						&left_bin_stats.entries[feature_index],
						left_sum_gradients,
						left_sum_hessians,
						left_examples_index_range.clone(),
						options,
					),
					find_best_continuous_split_for_feature_left_to_right(
						feature_index,
						binning_instructions,
						&right_bin_stats.entries[feature_index],
						right_sum_gradients,
						right_sum_hessians,
						right_examples_index_range.clone(),
						options,
					),
				),
				BinningInstructions::Enum { .. } => (
					find_best_discrete_split_for_feature_left_to_right(
						feature_index,
						&binning_instructions,
						&left_bin_stats.entries[feature_index],
						left_sum_gradients,
						left_sum_hessians,
						left_examples_index_range.clone(),
						options,
					),
					find_best_discrete_split_for_feature_left_to_right(
						feature_index,
						&binning_instructions,
						&right_bin_stats.entries[feature_index],
						right_sum_gradients,
						right_sum_hessians,
						right_examples_index_range.clone(),
						options,
					),
				),
			}
		})
		.collect();
	let (left, right) = best.into_iter().fold(
		(None, None),
		|a: (Option<ChooseBestSplitOutput>, Option<ChooseBestSplitOutput>),
		 b: (Option<ChooseBestSplitOutput>, Option<ChooseBestSplitOutput>)| {
			let left = match (a.0, b.0) {
				(Some(a), Some(b)) => {
					if a.gain > b.gain {
						Some(a)
					} else {
						Some(b)
					}
				}
				(Some(a), None) => Some(a),
				(None, Some(b)) => Some(b),
				(None, None) => None,
			};
			let right = match (a.1, b.1) {
				(Some(a), Some(b)) => {
					if a.gain > b.gain {
						Some(a)
					} else {
						Some(b)
					}
				}
				(Some(a), None) => Some(a),
				(None, Some(b)) => Some(b),
				(None, None) => None,
			};
			(left, right)
		},
	);
	(left, right)
}

/// Find the best split for this feature by iterating over the bins in sorted order, adding bins to the left tree and removing them from the right.
fn find_best_continuous_split_for_feature_left_to_right(
	feature_index: usize,
	binning_instructions: &BinningInstructions,
	bin_stats_for_feature: &[f64],
	sum_gradients_parent: f64,
	sum_hessians_parent: f64,
	examples_index_range: Range<usize>,
	options: &TrainOptions,
) -> Option<ChooseBestSplitOutput> {
	let negative_loss_parent_node = compute_negative_loss(
		sum_gradients_parent,
		sum_hessians_parent,
		options.l2_regularization,
	);
	let mut best_split_so_far: Option<ChooseBestSplitOutput> = None;
	let count_multiplier = examples_index_range.len() as f64 / sum_hessians_parent;
	let mut left_sum_gradients = 0.0;
	let mut left_sum_hessians = 0.0;
	let mut left_n_examples = 0;
	for (bin_index, bin_stats_entry) in bin_stats_for_feature[0..bin_stats_for_feature.len() - 2]
		.chunks(2)
		.enumerate()
	{
		let sum_gradients = bin_stats_entry[0];
		let sum_hessians = bin_stats_entry[1];
		left_n_examples += (sum_hessians * count_multiplier)
			.round()
			.to_usize()
			.unwrap();
		left_sum_hessians += sum_hessians;
		left_sum_gradients += sum_gradients;
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
		// Figure out whether invalid values should go to the left subtree or to the right when predicting depending on whether the training dataset contains missing values or not.
		let invalid_values_direction = if bin_stats_for_feature[1] > 0.0 {
			// there are missing values in the training dataset and they have been added to the left subtree
			SplitDirection::Left
		} else {
			// there are no missing values in the training dataset. missing values should go to the branch with more examples
			if left_n_examples >= right_n_examples {
				SplitDirection::Left
			} else {
				SplitDirection::Right
			}
		};
		let split = TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
			feature_index,
			bin_index: bin_index.to_u8().unwrap(),
			split_value: match binning_instructions {
				BinningInstructions::Number { thresholds } => match bin_index.checked_sub(1) {
					Some(i) => thresholds[i],
					None => f32::MIN,
				},
				_ => unreachable!(),
			},
			invalid_values_direction,
		});
		let current_split = ChooseBestSplitOutput {
			feature_index,
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
					best_split_so_far = Some(current_split);
				}
			}
			None => {
				if current_split.gain > options.min_gain_to_split {
					best_split_so_far = Some(current_split);
				}
			}
		}
	}
	best_split_so_far
}

/**
Find the best split for this discrete (categorical) feature. A discrete split is a partition of the categories into two subsets where one subset goes to the left subtree and one goes to the right.
To find the subsets:
1. Sort the bins by sum_gradients / (sum_hessians + categorical_smoothing_factor).
2. Perform the same algorithm to find the best split as the continuous setting, but iterate bins in the sorted order defined in step 1.
*/
fn find_best_discrete_split_for_feature_left_to_right(
	feature_index: usize,
	binning_instructions: &BinningInstructions,
	bin_stats_for_feature: &[f64],
	sum_gradients_parent: f64,
	sum_hessians_parent: f64,
	examples_index_range: Range<usize>,
	options: &TrainOptions,
) -> Option<ChooseBestSplitOutput> {
	let mut best_split_so_far: Option<ChooseBestSplitOutput> = None;
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
	let categorical_bin_score = |bin: &[f64]| bin[0] / (bin[1] + smoothing_factor);
	let mut sorted_bin_stats: Vec<(usize, &[f64])> =
		bin_stats_for_feature.chunks(2).enumerate().collect();
	sorted_bin_stats.sort_by(|(_, a), (_, b)| {
		categorical_bin_score(a)
			.partial_cmp(&categorical_bin_score(b))
			.unwrap()
	});
	let mut directions = vec![SplitDirection::Right; binning_instructions.n_valid_bins() + 1];
	let iter = sorted_bin_stats[0..sorted_bin_stats.len() - 1].iter();
	for (bin_index, bin_stats_entry) in iter {
		directions[*bin_index] = SplitDirection::Left;
		let sum_gradients = bin_stats_entry[0];
		let sum_hessians = bin_stats_entry[1];
		left_n_examples += (sum_hessians * count_multiplier)
			.round()
			.to_usize()
			.unwrap();
		left_sum_hessians += sum_hessians;
		left_sum_gradients += sum_gradients;
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
		let split = TrainBranchSplit::Discrete(TrainBranchSplitDiscrete {
			feature_index,
			directions: directions.clone(),
		});
		let current_split = ChooseBestSplitOutput {
			feature_index,
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
#[inline(always)]
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
#[inline(always)]
fn compute_negative_loss(sum_gradients: f64, sum_hessians: f64, l2_regularization: f32) -> f32 {
	((sum_gradients * sum_gradients) / (sum_hessians + l2_regularization.to_f64().unwrap()))
		.to_f32()
		.unwrap()
}
