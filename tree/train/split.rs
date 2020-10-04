use super::{
	bin_stats::{compute_bin_stats_for_feature_root, BinStats, BinStatsEntry},
	binning::{BinnedFeatures, BinningInstructions},
	TrainBranchSplit, TrainBranchSplitContinuous, TrainBranchSplitDiscrete,
};
use crate::{timing::Timing, SplitDirection, TrainOptions};
use num_traits::ToPrimitive;
use rayon::prelude::*;
use std::ops::Range;
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

pub struct BestSplitForFeature {
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
	let sum_gradients_root = gradients
		.par_iter()
		.map(|gradient| gradient.to_f64().unwrap())
		.sum::<f64>();
	let sum_hessians_root = if hessians_are_constant {
		gradients.len().to_f64().unwrap()
	} else {
		hessians
			.par_iter()
			.map(|hessian| hessian.to_f64().unwrap())
			.sum::<f64>()
	};
	#[cfg(feature = "timing")]
	timing.sum_gradients_and_hessians_root.inc(start.elapsed());

	// For each feature, compute bin stats and use them to choose the best split.
	let mut bin_stats = bin_stats_pool.get().unwrap();
	let best_split: Option<BestSplitForFeature> = pzip!(
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
			match binning_instructions {
				BinningInstructions::Number { .. } => choose_best_split_continuous(
					feature_index,
					&binning_instructions,
					bin_stats_for_feature,
					sum_gradients_root,
					sum_hessians_root,
					0..gradients.len(),
					options,
				),
				BinningInstructions::Enum { .. } => choose_best_split_discrete(
					feature_index,
					&binning_instructions,
					bin_stats_for_feature,
					sum_gradients_root,
					sum_hessians_root,
					0..gradients.len(),
					options,
				),
			}
		},
	)
	.filter_map(|split| split)
	.max_by(|a, b| a.gain.partial_cmp(&b.gain).unwrap());
	match best_split {
		Some(best_split) => ChooseBestSplitOutput::Success(ChooseBestSplitSuccess {
			gain: best_split.gain,
			split: best_split.split,
			sum_gradients: sum_gradients_root,
			sum_hessians: sum_hessians_root,
			left_n_examples: best_split.left_n_examples,
			left_sum_gradients: best_split.left_sum_gradients,
			left_sum_hessians: best_split.left_sum_hessians,
			right_n_examples: best_split.right_n_examples,
			right_sum_gradients: best_split.right_sum_gradients,
			right_sum_hessians: best_split.right_sum_hessians,
			bin_stats,
		}),
		None => ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
			sum_gradients: sum_gradients_root,
			sum_hessians: sum_hessians_root,
		}),
	}
}

#[allow(clippy::too_many_arguments)]
pub fn choose_best_splits_not_root(
	bin_stats_pool: &mut Pool<BinStats>,
	binned_features: &BinnedFeatures,
	gradients: &[f32],
	hessians: &[f32],
	left_child_examples_index: &[i32],
	right_child_examples_index: &[i32],
	gradients_ordered_buffer: &mut [f32],
	hessians_ordered_buffer: &mut [f32],
	parent_bin_stats: PoolGuard<BinStats>,
	hessians_are_constant: bool,
	options: &TrainOptions,
	#[cfg(feature = "timing")] timing: &Timing,
) -> (ChooseBestSplitOutput, ChooseBestSplitOutput) {
	// if !hessians_are_constant {
	// 		if examples_index_for_node.len() < 1024 {
	// 			izip!(
	// 				examples_index_for_node,
	// 				&mut *ordered_gradients,
	// 				&mut *ordered_hessians,
	// 			)
	// 			.for_each(
	// 				|(example_index, ordered_gradient, ordered_hessian)| unsafe {
	// 					*ordered_gradient = *gradients.get_unchecked(example_index.to_usize().unwrap());
	// 					*ordered_hessian = *hessians.get_unchecked(example_index.to_usize().unwrap());
	// 				},
	// 			);
	// 		} else {
	// 			let chunk_size = examples_index_for_node.len() / rayon::current_num_threads();
	// 			pzip!(
	// 				examples_index_for_node.par_chunks(chunk_size),
	// 				ordered_gradients.par_chunks_mut(chunk_size),
	// 				ordered_hessians.par_chunks_mut(chunk_size),
	// 			)
	// 			.for_each(
	// 				|(example_index_for_node, ordered_gradients, ordered_hessians)| {
	// 					izip!(example_index_for_node, ordered_gradients, ordered_hessians).for_each(
	// 						|(example_index, ordered_gradient, ordered_hessian)| unsafe {
	// 							*ordered_gradient =
	// 								*gradients.get_unchecked(example_index.to_usize().unwrap());
	// 							*ordered_hessian =
	// 								*hessians.get_unchecked(example_index.to_usize().unwrap());
	// 						},
	// 					);
	// 				},
	// 			);
	// 		}
	// 	} else {
	// 		if examples_index_for_node.len() < 1024 {
	// 			izip!(examples_index_for_node, &mut *ordered_gradients,).for_each(
	// 				|(example_index, ordered_gradient)| unsafe {
	// 					*ordered_gradient = *gradients.get_unchecked(example_index.to_usize().unwrap());
	// 				},
	// 			);
	// 		} else {
	// 			let chunk_size = examples_index_for_node.len() / rayon::current_num_threads();
	// 			pzip!(
	// 				examples_index_for_node.par_chunks(chunk_size),
	// 				ordered_gradients.par_chunks_mut(chunk_size),
	// 			)
	// 			.for_each(|(example_index_for_node, ordered_gradients)| unsafe {
	// 				izip!(example_index_for_node, ordered_gradients,).for_each(
	// 					|(example_index, ordered_gradient)| {
	// 						*ordered_gradient =
	// 							*gradients.get_unchecked(example_index.to_usize().unwrap());
	// 					},
	// 				);
	// 			});
	// 		}
	// 	}
	todo!()
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
) -> Option<BestSplitForFeature> {
	let mut best_split: Option<BestSplitForFeature> = None;
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
			let current_split = BestSplitForFeature {
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
) -> Option<BestSplitForFeature> {
	let mut best_split_so_far: Option<BestSplitForFeature> = None;
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
		let current_split = BestSplitForFeature {
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
