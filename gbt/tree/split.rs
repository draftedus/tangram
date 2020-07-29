use super::{
	super::{bin::BinInfo, types},
	bin_stats::{BinStats, BinStatsEntry},
	types::*,
};
use num_traits::ToPrimitive;
use rayon::prelude::*;
use std::ops::Range;

pub struct FindSplitOutput {
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

/// Find the split with the highest gain across all features,
/// if a valid one exists. A valid split will not exist if the split conditions
/// are violated for all potential splits.
pub fn find_split(
	bin_stats: &BinStats,
	include_features: &[bool],
	sum_gradients: f64,
	sum_hessians: f64,
	examples_index_range: Range<usize>,
	options: &types::TrainOptions,
) -> Option<FindSplitOutput> {
	(&bin_stats.entries, include_features, &bin_stats.bin_info)
		.into_par_iter()
		.enumerate()
		.filter_map(|(feature_index, (bin_stats, include_feature, bin_info))| {
			if !include_feature {
				return None;
			}
			match bin_info {
				BinInfo::Number { .. } => find_best_continuous_split_for_feature_left_to_right(
					feature_index,
					&bin_info,
					bin_stats,
					sum_gradients,
					sum_hessians,
					examples_index_range.clone(),
					options,
				),
				BinInfo::Enum { .. } => find_best_discrete_split_for_feature_left_to_right(
					feature_index,
					&bin_info,
					bin_stats,
					sum_gradients,
					sum_hessians,
					examples_index_range.clone(),
					options,
				),
			}
		})
		.max_by(|a, b| a.gain.partial_cmp(&b.gain).unwrap())
}

/// Find the split with the highest gain across all features
/// for both of the left and right child at the same time,
/// if a valid one exists. A valid split will not exist if the split conditions
/// are violated for all potential splits. By looping over the features once,
/// we increase the cache efficiency.
#[allow(clippy::too_many_arguments)]
pub fn find_split_both(
	left_bin_stats: &BinStats,
	left_sum_gradients: f64,
	left_sum_hessians: f64,
	left_examples_index_range: Range<usize>,
	right_bin_stats: &BinStats,
	right_sum_gradients: f64,
	right_sum_hessians: f64,
	right_examples_index_range: Range<usize>,
	include_features: &[bool],
	options: &types::TrainOptions,
) -> (Option<FindSplitOutput>, Option<FindSplitOutput>) {
	let best: Vec<(Option<FindSplitOutput>, Option<FindSplitOutput>)> =
		(0..left_bin_stats.entries.len())
			.into_par_iter()
			.map(|feature_index| {
				if !include_features[feature_index] {
					return (None, None);
				}
				let bin_info = &left_bin_stats.bin_info[feature_index];
				match bin_info {
					BinInfo::Number { .. } => (
						find_best_continuous_split_for_feature_left_to_right(
							feature_index,
							bin_info,
							&left_bin_stats.entries[feature_index],
							left_sum_gradients,
							left_sum_hessians,
							left_examples_index_range.clone(),
							options,
						),
						find_best_continuous_split_for_feature_left_to_right(
							feature_index,
							bin_info,
							&right_bin_stats.entries[feature_index],
							right_sum_gradients,
							right_sum_hessians,
							right_examples_index_range.clone(),
							options,
						),
					),
					BinInfo::Enum { .. } => (
						find_best_discrete_split_for_feature_left_to_right(
							feature_index,
							&bin_info,
							&left_bin_stats.entries[feature_index],
							left_sum_gradients,
							left_sum_hessians,
							left_examples_index_range.clone(),
							options,
						),
						find_best_discrete_split_for_feature_left_to_right(
							feature_index,
							&bin_info,
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
		|a: (Option<FindSplitOutput>, Option<FindSplitOutput>),
		 b: (Option<FindSplitOutput>, Option<FindSplitOutput>)| {
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

/// Find the best split for this feature by iterating over the bins in sorted order,
/// adding bins to the left tree and removing them from the right.
pub fn find_best_continuous_split_for_feature_left_to_right(
	feature_index: usize,
	bin_info: &BinInfo,
	bin_stats_for_feature: &[BinStatsEntry],
	sum_gradients_parent: f64,
	sum_hessians_parent: f64,
	examples_index_range: Range<usize>,
	options: &types::TrainOptions,
) -> Option<FindSplitOutput> {
	let negative_loss_parent_node = negative_loss(
		sum_gradients_parent,
		sum_hessians_parent,
		options.l2_regularization,
	);
	let mut best_split_so_far: Option<FindSplitOutput> = None;

	let mut left_sum_gradients = 0.0;
	let mut left_sum_hessians = 0.0;
	let mut left_n_examples = 0;

	for (bin_index, bin_stats_entry) in bin_stats_for_feature.iter().enumerate() {
		left_n_examples += bin_stats_entry.count;
		left_sum_hessians += bin_stats_entry.sum_hessians;
		left_sum_gradients += bin_stats_entry.sum_gradients;

		let right_sum_gradients = sum_gradients_parent - left_sum_gradients;
		let right_sum_hessians = sum_hessians_parent - left_sum_hessians;
		let right_n_examples = examples_index_range.len() - left_n_examples;

		// check if we have violated the min samples leaf constraint
		if left_n_examples < options.min_examples_leaf {
			continue;
		}
		if right_n_examples < options.min_examples_leaf {
			// since we are in left to right mode, we will only get less examples if we continue so break instead
			break;
		}

		if left_sum_hessians < options.min_sum_hessians_in_leaf as f64 {
			// Hessians are positive so the left sum hessians will continue to increase,
			// we can continue.
			continue;
		}
		if right_sum_hessians < options.min_sum_hessians_in_leaf as f64 {
			// Hessians are positive so we will continue to violate the min_hessian_to_split
			// condition for the right node, break.
			break;
		}

		let current_split_gain = gain(
			left_sum_gradients,
			left_sum_hessians,
			right_sum_gradients,
			right_sum_hessians,
			negative_loss_parent_node,
			options.l2_regularization,
		);

		let invalid_values_direction = if bin_stats_for_feature.first().unwrap().count > 0 {
			// we are in the function that splits from left to right
			types::SplitDirection::Left
		} else {
			// there are no missing values, we take the branch with more examples
			if left_n_examples >= right_n_examples {
				types::SplitDirection::Left
			} else {
				types::SplitDirection::Right
			}
		};

		let split = TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
			feature_index,
			bin_index: bin_index as u8,
			split_value: match bin_info {
				BinInfo::Number { thresholds } => {
					match bin_index.checked_sub(1) {
						Some(i) => thresholds[i],
						// its the null bucket
						None => f32::MIN,
					}
				}
				_ => unreachable!(),
			},
			invalid_values_direction,
		});

		let current_split = FindSplitOutput {
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

/// Find the best split for this discrete (categorical) feature.
/// A discrete split is a partition of the categories into two subsets where one subset goes to the
/// left subtree and one goes to the right.
/// To find the subsets:
///   1. Sort the bins by sum_gradients / (sum_hessians + categorical_smoothing_factor).
///   2. Perform the same algorithm to find the best split as the continuous setting, but iterate
///      bins in the sorted order defined in step 1.
pub fn find_best_discrete_split_for_feature_left_to_right(
	feature_index: usize,
	bin_info: &BinInfo,
	bin_stats_for_feature: &[BinStatsEntry],
	sum_gradients: f64,
	sum_hessians: f64,
	examples_index_range: Range<usize>,
	options: &types::TrainOptions,
) -> Option<FindSplitOutput> {
	let mut best_split_so_far: Option<FindSplitOutput> = None;

	let l2_regularization = options.l2_regularization + options.discrete_l2_regularization;

	let negative_loss_parent_node = negative_loss(sum_gradients, sum_hessians, l2_regularization);

	let mut left_sum_gradients = 0.0;
	let mut left_sum_hessians = 0.0;
	let mut left_n_examples = 0;

	let categorical_bin_score = |bin: &BinStatsEntry| {
		bin.sum_gradients / (bin.sum_hessians + options.discrete_smoothing_factor as f64)
	};
	let mut sorted_bin_stats: Vec<(usize, &BinStatsEntry)> = (0..bin_stats_for_feature.len())
		.zip(bin_stats_for_feature.iter())
		.collect();
	sorted_bin_stats.sort_by(|(_, a), (_, b)| {
		categorical_bin_score(a)
			.partial_cmp(&categorical_bin_score(b))
			.unwrap()
	});

	let mut directions = types::BinDirections::new(bin_info.n_valid_bins() + 1, true);

	for (bin_index, bin_stats_entry) in sorted_bin_stats.iter() {
		directions.set(*bin_index as u8, false);
		left_n_examples += bin_stats_entry.count;
		left_sum_hessians += bin_stats_entry.sum_hessians;
		left_sum_gradients += bin_stats_entry.sum_gradients;

		let right_sum_gradients = sum_gradients - left_sum_gradients;
		let right_sum_hessians = sum_hessians - left_sum_hessians;
		let right_n_examples = examples_index_range.len() - left_n_examples;

		// check if we have violated the min samples leaf constraint
		if left_n_examples < options.min_examples_leaf {
			continue;
		}
		// check if we have violated the min examples per categorical branch constraint
		if left_n_examples < options.discrete_min_examples_per_branch {
			continue;
		}

		if right_n_examples < options.min_examples_leaf {
			// since we are in left to right mode, we will only get less examples if we continue so break instead
			break;
		}
		if right_n_examples < options.discrete_min_examples_per_branch {
			// since we are in left to right mode, we will only get less examples if we continue so break instead
			break;
		}

		if left_sum_hessians < options.min_sum_hessians_in_leaf as f64 {
			// Hessians are positive so the left sum hessians will continue to increase,
			// we can continue.
			continue;
		}
		if right_sum_hessians < options.min_sum_hessians_in_leaf as f64 {
			// Hessians are positive so we will continue to violate the min_hessian_to_split
			// condition for the right node, break.
			break;
		}

		let current_split_gain = gain(
			left_sum_gradients,
			left_sum_hessians,
			right_sum_gradients,
			right_sum_hessians,
			negative_loss_parent_node,
			l2_regularization,
		);

		let invalid_values_direction = if bin_stats_for_feature.first().unwrap().count > 0 {
			// we are in the function that splits from left to right
			types::SplitDirection::Left
		} else {
			// there are no missing values, we take the branch with more examples
			if left_n_examples >= right_n_examples {
				types::SplitDirection::Left
			} else {
				types::SplitDirection::Right
			}
		};

		let split = TrainBranchSplit::Discrete(TrainBranchSplitDiscrete {
			feature_index,
			directions: directions.clone(),
			invalid_values_direction,
		});

		let current_split = FindSplitOutput {
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

#[inline(always)]
pub fn gain(
	sum_gradients_left: f64,
	sum_hessians_left: f64,
	sum_gradients_right: f64,
	sum_hessians_right: f64,
	negative_loss_current_node: f32,
	l2_regularization: f32,
) -> f32 {
	let left = negative_loss(sum_gradients_left, sum_hessians_left, l2_regularization);
	let right = negative_loss(sum_gradients_right, sum_hessians_right, l2_regularization);
	left + right - negative_loss_current_node
}

#[inline(always)]
pub fn negative_loss(sum_gradients: f64, sum_hessians: f64, l2_regularization: f32) -> f32 {
	((sum_gradients * sum_gradients) / (sum_hessians + l2_regularization.to_f64().unwrap()))
		.to_f32()
		.unwrap()
}
