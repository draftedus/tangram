#[cfg(feature = "timing")]
use crate::timing::Timing;
use crate::{
	compute_bin_stats::{
		compute_bin_stats_column_major_not_root, compute_bin_stats_column_major_root,
		compute_bin_stats_row_major_not_root, compute_bin_stats_row_major_root,
		compute_bin_stats_subtraction, BinStats, BinStatsEntry,
	},
	compute_binned_features::{BinnedFeaturesColumnMajor, BinnedFeaturesRowMajor},
	compute_binning_instructions::BinningInstruction,
	train_tree::{TrainBranchSplit, TrainBranchSplitContinuous, TrainBranchSplitDiscrete},
	BinnedFeaturesLayout, SplitDirection, TrainOptions,
};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use rayon::prelude::*;
use tangram_pool::{Pool, PoolItem};
use tangram_thread_pool::pzip;

pub struct ChooseBestSplitRootOptions<'a> {
	pub bin_stats_pool: &'a Pool<BinStats>,
	pub binned_features_column_major: &'a BinnedFeaturesColumnMajor,
	pub binned_features_row_major: &'a BinnedFeaturesRowMajor,
	pub binning_instructions: &'a [BinningInstruction],
	pub gradients: &'a [f32],
	pub hessians_are_constant: bool,
	pub hessians: &'a [f32],
	#[cfg(feature = "timing")]
	pub timing: &'a Timing,
	pub train_options: &'a TrainOptions,
}

pub struct ChooseBestSplitsNotRootOptions<'a> {
	pub bin_stats_pool: &'a Pool<BinStats>,
	pub binned_features_column_major: &'a BinnedFeaturesColumnMajor,
	pub binned_features_row_major: &'a BinnedFeaturesRowMajor,
	pub binning_instructions: &'a [BinningInstruction],
	pub gradients_ordered_buffer: &'a mut [f32],
	pub gradients: &'a [f32],
	pub hessians_are_constant: bool,
	pub hessians_ordered_buffer: &'a mut [f32],
	pub hessians: &'a [f32],
	pub left_child_examples_index: &'a [i32],
	pub left_child_n_examples: usize,
	pub left_child_sum_gradients: f64,
	pub left_child_sum_hessians: f64,
	pub parent_bin_stats: PoolItem<BinStats>,
	pub parent_depth: usize,
	pub right_child_examples_index: &'a [i32],
	pub right_child_n_examples: usize,
	pub right_child_sum_gradients: f64,
	pub right_child_sum_hessians: f64,
	#[cfg(feature = "timing")]
	pub timing: &'a Timing,
	pub train_options: &'a TrainOptions,
}

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
	pub bin_stats: PoolItem<BinStats>,
}

pub struct ChooseBestSplitFailure {
	pub sum_gradients: f64,
	pub sum_hessians: f64,
}

pub struct ChooseBestSplitForFeatureOutput {
	pub gain: f32,
	pub split: TrainBranchSplit,
	pub left_approximate_n_examples: usize,
	pub left_sum_gradients: f64,
	pub left_sum_hessians: f64,
	pub right_approximate_n_examples: usize,
	pub right_sum_gradients: f64,
	pub right_sum_hessians: f64,
}

const MIN_EXAMPLES_TO_PARALLELIZE: usize = 1024;

pub fn choose_best_split_root(options: ChooseBestSplitRootOptions) -> ChooseBestSplitOutput {
	let ChooseBestSplitRootOptions {
		bin_stats_pool,
		binned_features_column_major,
		binned_features_row_major,
		binning_instructions,
		gradients,
		hessians_are_constant,
		hessians,
		train_options,
		..
	} = options;
	#[cfg(feature = "timing")]
	let timing = options.timing;
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
	let should_try_to_split_root = gradients.len() >= 2 * train_options.min_examples_per_node
		&& sum_hessians >= 2.0 * train_options.min_sum_hessians_per_node.to_f64().unwrap();
	if !should_try_to_split_root {
		return ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
			sum_gradients,
			sum_hessians,
		});
	}

	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	// For each feature, compute bin stats and use them to choose the best split.
	let mut bin_stats = bin_stats_pool.get().unwrap();
	let best_split_output: Option<ChooseBestSplitForFeatureOutput> =
		match train_options.binned_features_layout {
			BinnedFeaturesLayout::ColumnMajor => {
				let bin_stats = bin_stats.as_column_major_mut().unwrap();
				choose_best_split_root_column_major(ChooseBestSplitRootColumnMajorOptions {
					bin_stats,
					binned_features_column_major,
					binning_instructions,
					gradients,
					hessians_are_constant,
					hessians,
					sum_gradients,
					sum_hessians,
					train_options,
				})
			}
			BinnedFeaturesLayout::RowMajor => {
				let bin_stats = bin_stats.as_row_major_mut().unwrap();
				choose_best_split_root_row_major(ChooseBestSplitRootRowMajorOptions {
					bin_stats,
					binned_features_row_major,
					binning_instructions,
					gradients,
					hessians_are_constant,
					hessians,
					sum_gradients,
					sum_hessians,
					train_options,
				})
			}
		};
	#[cfg(feature = "timing")]
	timing.choose_best_split_root.inc(start.elapsed());

	// Assemble the output.
	match best_split_output {
		Some(best_split) => ChooseBestSplitOutput::Success(ChooseBestSplitSuccess {
			gain: best_split.gain,
			split: best_split.split,
			sum_gradients,
			sum_hessians,
			left_n_examples: best_split.left_approximate_n_examples,
			left_sum_gradients: best_split.left_sum_gradients,
			left_sum_hessians: best_split.left_sum_hessians,
			right_n_examples: best_split.right_approximate_n_examples,
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

struct ChooseBestSplitRootColumnMajorOptions<'a> {
	bin_stats: &'a mut Vec<Vec<BinStatsEntry>>,
	binned_features_column_major: &'a BinnedFeaturesColumnMajor,
	binning_instructions: &'a [BinningInstruction],
	gradients: &'a [f32],
	hessians_are_constant: bool,
	hessians: &'a [f32],
	sum_gradients: f64,
	sum_hessians: f64,
	train_options: &'a TrainOptions,
}

fn choose_best_split_root_column_major(
	options: ChooseBestSplitRootColumnMajorOptions,
) -> Option<ChooseBestSplitForFeatureOutput> {
	let ChooseBestSplitRootColumnMajorOptions {
		bin_stats,
		binned_features_column_major,
		binning_instructions,
		gradients,
		hessians_are_constant,
		hessians,
		sum_gradients,
		sum_hessians,
		train_options,
	} = options;
	pzip!(
		binning_instructions,
		&binned_features_column_major.columns,
		bin_stats
	)
	.enumerate()
	.map(
		|(feature_index, (binning_instructions, binned_feature_column, bin_stats_for_feature))| {
			// Compute the bin stats.
			compute_bin_stats_column_major_root(
				bin_stats_for_feature,
				binned_feature_column,
				gradients,
				hessians,
				hessians_are_constant,
			);
			// Choose the best split for this featue.
			choose_best_split_for_feature(
				feature_index,
				binning_instructions,
				bin_stats_for_feature,
				binned_feature_column.len(),
				sum_gradients,
				sum_hessians,
				train_options,
			)
		},
	)
	.filter_map(|split| split)
	.max_by(|a, b| a.gain.partial_cmp(&b.gain).unwrap())
}

struct ChooseBestSplitRootRowMajorOptions<'a> {
	bin_stats: &'a mut Vec<BinStatsEntry>,
	binned_features_row_major: &'a BinnedFeaturesRowMajor,
	binning_instructions: &'a [BinningInstruction],
	gradients: &'a [f32],
	hessians_are_constant: bool,
	hessians: &'a [f32],
	sum_gradients: f64,
	sum_hessians: f64,
	train_options: &'a TrainOptions,
}

fn choose_best_split_root_row_major(
	options: ChooseBestSplitRootRowMajorOptions,
) -> Option<ChooseBestSplitForFeatureOutput> {
	let ChooseBestSplitRootRowMajorOptions {
		bin_stats,
		binned_features_row_major,
		binning_instructions,
		gradients,
		hessians_are_constant,
		hessians,
		sum_gradients,
		sum_hessians,
		train_options,
	} = options;
	// Compute the bin stats for the child with fewer examples.
	let n_examples = binned_features_row_major.values_with_offsets.nrows();
	let n_threads = rayon::current_num_threads();
	let chunk_size = (n_examples + n_threads - 1) / n_threads;
	*bin_stats = pzip!(
		binned_features_row_major
			.values_with_offsets
			.axis_chunks_iter(Axis(0), chunk_size),
		gradients.par_chunks(chunk_size),
		hessians.par_chunks(chunk_size)
	)
	.map(|(binned_features_chunk, gradients_chunk, hessians_chunk)| {
		let mut bin_stats_chunk: Vec<BinStatsEntry> =
			bin_stats.iter().map(|_| BinStatsEntry::default()).collect();
		compute_bin_stats_row_major_root(
			bin_stats_chunk.as_mut_slice(),
			binned_features_chunk,
			gradients_chunk,
			hessians_chunk,
			hessians_are_constant,
		);
		bin_stats_chunk
	})
	.reduce(
		|| bin_stats.iter().map(|_| BinStatsEntry::default()).collect(),
		|mut res, chunk| {
			res.iter_mut().zip(chunk.iter()).for_each(|(res, chunk)| {
				res.sum_gradients += chunk.sum_gradients;
				res.sum_hessians += chunk.sum_hessians;
			});
			res
		},
	);
	// Choose the best split for each featue.
	let bin_stats = super_unsafe::SuperUnsafe::new(bin_stats);
	pzip!(binning_instructions, &binned_features_row_major.offsets)
		.enumerate()
		.map(|(feature_index, (binning_instructions, offset))| {
			let offset = offset.to_usize().unwrap();
			let bin_stats_range = offset..offset + binning_instructions.n_bins();
			let bin_stats_for_feature = unsafe { &mut bin_stats.get()[bin_stats_range] };
			choose_best_split_for_feature(
				feature_index,
				binning_instructions,
				bin_stats_for_feature,
				n_examples,
				sum_gradients,
				sum_hessians,
				train_options,
			)
		})
		.filter_map(|split| split)
		.max_by(|a, b| a.gain.partial_cmp(&b.gain).unwrap())
}

pub fn choose_best_splits_not_root(
	options: ChooseBestSplitsNotRootOptions,
) -> (ChooseBestSplitOutput, ChooseBestSplitOutput) {
	let ChooseBestSplitsNotRootOptions {
		bin_stats_pool,
		binned_features_column_major,
		binned_features_row_major,
		binning_instructions,
		gradients_ordered_buffer,
		gradients,
		hessians_are_constant,
		hessians_ordered_buffer,
		hessians,
		left_child_examples_index,
		left_child_n_examples,
		left_child_sum_gradients,
		left_child_sum_hessians,
		parent_bin_stats,
		parent_depth,
		right_child_examples_index,
		right_child_n_examples,
		right_child_sum_gradients,
		right_child_sum_hessians,
		train_options,
		..
	} = options;
	let mut left_child_output = ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
		sum_gradients: left_child_sum_gradients,
		sum_hessians: left_child_sum_hessians,
	});
	let mut right_child_output = ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
		sum_gradients: right_child_sum_gradients,
		sum_hessians: right_child_sum_hessians,
	});

	// Determine if we should try to split the left and/or right children of this branch.
	let children_will_exceed_max_depth = if let Some(max_depth) = train_options.max_depth {
		parent_depth + 1 > max_depth - 1
	} else {
		false
	};
	let should_try_to_split_left_child = !children_will_exceed_max_depth
		&& left_child_examples_index.len() >= train_options.min_examples_per_node * 2;
	let should_try_to_split_right_child = !children_will_exceed_max_depth
		&& right_child_examples_index.len() >= train_options.min_examples_per_node * 2;

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

	// If the binned features are column major, fill the gradients and hessians ordered buffers. The buffers contain the gradients and hessians for each example as ordered by the examples index. This makes the access of the gradients and hessians sequential in the next step.
	if let BinnedFeaturesLayout::ColumnMajor = train_options.binned_features_layout {
		fill_gradients_and_hessians_ordered_buffers(
			smaller_child_examples_index,
			gradients,
			hessians,
			gradients_ordered_buffer,
			hessians_ordered_buffer,
			hessians_are_constant,
		);
	}

	// Collect the best splits for the left and right children for each feature.
	let children_best_splits_for_features: Vec<(
		Option<ChooseBestSplitForFeatureOutput>,
		Option<ChooseBestSplitForFeatureOutput>,
	)> = match train_options.binned_features_layout {
		BinnedFeaturesLayout::RowMajor => {
			let smaller_child_bin_stats = smaller_child_bin_stats.as_row_major_mut().unwrap();
			let larger_child_bin_stats = larger_child_bin_stats.as_row_major_mut().unwrap();
			choose_best_splits_not_root_row_major(ChooseBestSplitsNotRootRowMajorOptions {
				should_try_to_split_right_child,
				smaller_child_bin_stats,
				larger_child_bin_stats,
				binned_features_row_major,
				binning_instructions,
				gradients,
				hessians_are_constant,
				hessians,
				train_options,
				left_child_n_examples,
				left_child_sum_gradients,
				left_child_sum_hessians,
				right_child_n_examples,
				right_child_sum_gradients,
				right_child_sum_hessians,
				smaller_child_examples_index,
				should_try_to_split_left_child,
				smaller_child_direction,
			})
		}
		BinnedFeaturesLayout::ColumnMajor => {
			let smaller_child_bin_stats = smaller_child_bin_stats.as_column_major_mut().unwrap();
			let larger_child_bin_stats = larger_child_bin_stats.as_column_major_mut().unwrap();
			choose_best_splits_not_root_column_major(ChooseBestSplitsNotRootColumnMajorOptions {
				should_try_to_split_right_child,
				smaller_child_bin_stats,
				larger_child_bin_stats,
				binned_features_column_major,
				binning_instructions,
				gradients_ordered_buffer,
				hessians_are_constant,
				hessians_ordered_buffer,
				train_options,
				left_child_n_examples,
				left_child_sum_gradients,
				left_child_sum_hessians,
				right_child_n_examples,
				right_child_sum_gradients,
				right_child_sum_hessians,
				smaller_child_examples_index,
				should_try_to_split_left_child,
				smaller_child_direction,
			})
		}
	};

	// Choose the splits for the left and right children with the highest gain.
	let (left_child_best_split, right_child_best_split) =
		choose_splits_with_highest_gain(children_best_splits_for_features);

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
			left_n_examples: best_split.left_approximate_n_examples,
			left_sum_gradients: best_split.left_sum_gradients,
			left_sum_hessians: best_split.left_sum_hessians,
			right_n_examples: best_split.right_approximate_n_examples,
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
			left_n_examples: best_split.left_approximate_n_examples,
			left_sum_gradients: best_split.left_sum_gradients,
			left_sum_hessians: best_split.left_sum_hessians,
			right_n_examples: best_split.right_approximate_n_examples,
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

struct ChooseBestSplitsNotRootColumnMajorOptions<'a> {
	binned_features_column_major: &'a BinnedFeaturesColumnMajor,
	binning_instructions: &'a [BinningInstruction],
	gradients_ordered_buffer: &'a [f32],
	hessians_are_constant: bool,
	hessians_ordered_buffer: &'a [f32],
	larger_child_bin_stats: &'a mut Vec<Vec<BinStatsEntry>>,
	left_child_n_examples: usize,
	left_child_sum_gradients: f64,
	left_child_sum_hessians: f64,
	right_child_n_examples: usize,
	right_child_sum_gradients: f64,
	right_child_sum_hessians: f64,
	should_try_to_split_left_child: bool,
	should_try_to_split_right_child: bool,
	smaller_child_bin_stats: &'a mut Vec<Vec<BinStatsEntry>>,
	smaller_child_direction: SplitDirection,
	smaller_child_examples_index: &'a [i32],
	train_options: &'a TrainOptions,
}

fn choose_best_splits_not_root_column_major(
	options: ChooseBestSplitsNotRootColumnMajorOptions,
) -> Vec<(
	Option<ChooseBestSplitForFeatureOutput>,
	Option<ChooseBestSplitForFeatureOutput>,
)> {
	let ChooseBestSplitsNotRootColumnMajorOptions {
		binned_features_column_major,
		binning_instructions,
		gradients_ordered_buffer,
		hessians_are_constant,
		hessians_ordered_buffer,
		larger_child_bin_stats,
		left_child_n_examples,
		left_child_sum_gradients,
		left_child_sum_hessians,
		right_child_n_examples,
		right_child_sum_gradients,
		right_child_sum_hessians,
		should_try_to_split_left_child,
		should_try_to_split_right_child,
		smaller_child_bin_stats,
		smaller_child_direction,
		smaller_child_examples_index,
		train_options,
	} = options;
	pzip!(
		binning_instructions,
		&binned_features_column_major.columns,
		smaller_child_bin_stats,
		larger_child_bin_stats,
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
			compute_bin_stats_column_major_not_root(
				smaller_child_bin_stats_for_feature,
				smaller_child_examples_index,
				binned_features_column,
				gradients_ordered_buffer,
				hessians_ordered_buffer,
				hessians_are_constant,
			);
			// Compute the larger child bin stats by subtraction.
			compute_bin_stats_subtraction(
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
					train_options,
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
					train_options,
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
	.collect()
}

struct ChooseBestSplitsNotRootRowMajorOptions<'a> {
	binned_features_row_major: &'a BinnedFeaturesRowMajor,
	binning_instructions: &'a [BinningInstruction],
	gradients: &'a [f32],
	hessians_are_constant: bool,
	hessians: &'a [f32],
	larger_child_bin_stats: &'a mut Vec<BinStatsEntry>,
	left_child_n_examples: usize,
	left_child_sum_gradients: f64,
	left_child_sum_hessians: f64,
	right_child_n_examples: usize,
	right_child_sum_gradients: f64,
	right_child_sum_hessians: f64,
	should_try_to_split_left_child: bool,
	should_try_to_split_right_child: bool,
	smaller_child_bin_stats: &'a mut Vec<BinStatsEntry>,
	smaller_child_direction: SplitDirection,
	smaller_child_examples_index: &'a [i32],
	train_options: &'a TrainOptions,
}

fn choose_best_splits_not_root_row_major(
	options: ChooseBestSplitsNotRootRowMajorOptions,
) -> Vec<(
	Option<ChooseBestSplitForFeatureOutput>,
	Option<ChooseBestSplitForFeatureOutput>,
)> {
	let ChooseBestSplitsNotRootRowMajorOptions {
		binned_features_row_major,
		binning_instructions,
		gradients,
		hessians_are_constant,
		hessians,
		larger_child_bin_stats,
		left_child_n_examples,
		left_child_sum_gradients,
		left_child_sum_hessians,
		right_child_n_examples,
		right_child_sum_gradients,
		right_child_sum_hessians,
		should_try_to_split_left_child,
		should_try_to_split_right_child,
		smaller_child_bin_stats,
		smaller_child_direction,
		smaller_child_examples_index,
		train_options,
	} = options;
	// Compute the bin stats for the child with fewer examples.
	let smaller_child_n_examples = smaller_child_examples_index.len();
	if smaller_child_n_examples < MIN_EXAMPLES_TO_PARALLELIZE {
		compute_bin_stats_row_major_not_root(
			smaller_child_bin_stats.as_mut_slice(),
			smaller_child_examples_index,
			binned_features_row_major,
			gradients,
			hessians,
			hessians_are_constant,
		);
	} else {
		let n_threads = rayon::current_num_threads();
		let chunk_size = (smaller_child_n_examples + n_threads - 1) / n_threads;
		*smaller_child_bin_stats = pzip!(smaller_child_examples_index.par_chunks(chunk_size))
			.map(|(smaller_child_examples_index_chunk,)| {
				let mut smaller_child_bin_stats_chunk: Vec<BinStatsEntry> = smaller_child_bin_stats
					.iter()
					.map(|_| BinStatsEntry::default())
					.collect();
				compute_bin_stats_row_major_not_root(
					smaller_child_bin_stats_chunk.as_mut_slice(),
					smaller_child_examples_index_chunk,
					binned_features_row_major,
					gradients,
					hessians,
					hessians_are_constant,
				);
				smaller_child_bin_stats_chunk
			})
			.reduce(
				|| {
					smaller_child_bin_stats
						.iter()
						.map(|_| BinStatsEntry::default())
						.collect()
				},
				|mut res, chunk| {
					res.iter_mut().zip(chunk.iter()).for_each(|(res, chunk)| {
						res.sum_gradients += chunk.sum_gradients;
						res.sum_hessians += chunk.sum_hessians;
					});
					res
				},
			);
	}
	// Choose the best split for each feature.
	let smaller_child_bin_stats = super_unsafe::SuperUnsafe::new(smaller_child_bin_stats);
	let larger_child_bin_stats = super_unsafe::SuperUnsafe::new(larger_child_bin_stats);
	pzip!(binning_instructions, &binned_features_row_major.offsets)
		.enumerate()
		.map(|(feature_index, (binning_instructions, offset))| {
			let smaller_child_bin_stats_for_feature = unsafe {
				&mut smaller_child_bin_stats.get()[offset.to_usize().unwrap()
					..offset.to_usize().unwrap() + binning_instructions.n_bins()]
			};
			let larger_child_bin_stats_for_feature = unsafe {
				&mut larger_child_bin_stats.get()[offset.to_usize().unwrap()
					..offset.to_usize().unwrap() + binning_instructions.n_bins()]
			};
			// Compute the larger child bin stats by subtraction.
			compute_bin_stats_subtraction(
				smaller_child_bin_stats_for_feature,
				larger_child_bin_stats_for_feature,
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
					train_options,
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
					train_options,
				)
			} else {
				None
			};
			(
				left_child_best_split_for_feature,
				right_child_best_split_for_feature,
			)
		})
		.collect()
}

/// Choose the best split for a feature by choosing a continuous split for number features and a discrete split for enum features.
fn choose_best_split_for_feature(
	feature_index: usize,
	binning_instructions: &BinningInstruction,
	bin_stats_for_feature: &[BinStatsEntry],
	n_examples: usize,
	sum_gradients: f64,
	sum_hessians: f64,
	train_options: &TrainOptions,
) -> Option<ChooseBestSplitForFeatureOutput> {
	match binning_instructions {
		BinningInstruction::Number { .. } => choose_best_split_for_continuous_feature(
			feature_index,
			&binning_instructions,
			bin_stats_for_feature,
			n_examples,
			sum_gradients,
			sum_hessians,
			train_options,
		),
		BinningInstruction::Enum { .. } => choose_best_split_for_discrete_feature(
			feature_index,
			&binning_instructions,
			bin_stats_for_feature,
			n_examples,
			sum_gradients,
			sum_hessians,
			train_options,
		),
	}
}

/// Choose the best continuous split for this feature.
fn choose_best_split_for_continuous_feature(
	feature_index: usize,
	binning_instructions: &BinningInstruction,
	bin_stats_for_feature: &[BinStatsEntry],
	n_examples_parent: usize,
	sum_gradients_parent: f64,
	sum_hessians_parent: f64,
	train_options: &TrainOptions,
) -> Option<ChooseBestSplitForFeatureOutput> {
	let mut best_split_for_feature: Option<ChooseBestSplitForFeatureOutput> = None;
	let l2_regularization = train_options.l2_regularization;
	let negative_loss_for_parent_node =
		compute_negative_loss(sum_gradients_parent, sum_hessians_parent, l2_regularization);
	let mut left_approximate_n_examples = 0;
	let mut left_sum_gradients = 0.0;
	let mut left_sum_hessians = 0.0;
	let thresholds = match binning_instructions {
		BinningInstruction::Number { thresholds } => thresholds,
		_ => unreachable!(),
	};
	// Always send invalid values to the left.
	let invalid_values_direction = SplitDirection::Left;
	let invalid_bin_stats = bin_stats_for_feature.get(0).unwrap().clone();
	left_sum_gradients += invalid_bin_stats.sum_gradients;
	left_sum_hessians += invalid_bin_stats.sum_hessians;
	// For each bin, determine if splitting at that bin's value would produce a better split.
	for (valid_bin_index, bin_stats_entry) in bin_stats_for_feature
		[1..bin_stats_for_feature.len() - 1]
		.iter()
		.enumerate()
	{
		// Approximate the number of examples that would be sent to the left child by assuming the fraction of examples is equal to the fraction of the sum of hessians.
		left_approximate_n_examples += (bin_stats_entry.sum_hessians
			* n_examples_parent.to_f64().unwrap()
			/ sum_hessians_parent)
			.round()
			.to_usize()
			.unwrap();
		left_sum_gradients += bin_stats_entry.sum_gradients;
		left_sum_hessians += bin_stats_entry.sum_hessians;
		// Above we approximate the number of examples based on the sum of hessians. It is possible this approximation is off by enough that left_approximate_n_examples exceeds n_examples_parent. If this happens, we must not consider any further bins as the split value by exiting the loop.
		let right_approximate_n_examples =
			match n_examples_parent.checked_sub(left_approximate_n_examples) {
				Some(right_n_examples) => right_n_examples,
				None => break,
			};
		// Compute the sum of gradients and hessians for the examples that would be sent to the right by this split. To make this fast, subtract the values for the left child from the parent.
		let right_sum_gradients = sum_gradients_parent - left_sum_gradients;
		let right_sum_hessians = sum_hessians_parent - left_sum_hessians;
		// Check if fewer than `min_examples_per_node` would be sent to the left child by this split.
		if left_approximate_n_examples < train_options.min_examples_per_node {
			continue;
		}
		// Check if fewer than `min_examples_per_node` would be sent to the right child by this split. If true, then splitting by the thresholds for all subsequent bins will also fail, so we can exit the loop.
		if right_approximate_n_examples < train_options.min_examples_per_node {
			break;
		}
		// Check if the sum of hessians for examples that would be sent to the left child by this split falls below `min_sum_hessians_per_node`.
		if left_sum_hessians < train_options.min_sum_hessians_per_node.to_f64().unwrap() {
			continue;
		}
		// Check if the sum of hessians for examples that would be sent to the right child by this split falls below `min_sum_hessians_per_node`. If true, then splitting by the thresholds for all subsequent bins will also fail, so we can exit the loop. This is true because hessians are always positive.
		if right_sum_hessians < train_options.min_sum_hessians_per_node.to_f64().unwrap() {
			break;
		}
		// Compute the gain for this candidate split.
		let gain = compute_gain(
			left_sum_gradients,
			left_sum_hessians,
			right_sum_gradients,
			right_sum_hessians,
			negative_loss_for_parent_node,
			l2_regularization,
		);
		// If this split has a higher gain or if there is no existing best split, then use this split.
		if best_split_for_feature
			.as_ref()
			.map(|best_split_for_feature| gain > best_split_for_feature.gain)
			.unwrap_or(true)
		{
			let split_value = *thresholds.get(valid_bin_index).unwrap();
			let split = TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
				feature_index,
				bin_index: valid_bin_index + 1,
				split_value,
				invalid_values_direction,
			});
			best_split_for_feature = Some(ChooseBestSplitForFeatureOutput {
				gain,
				split,
				left_approximate_n_examples,
				left_sum_gradients,
				left_sum_hessians,
				right_approximate_n_examples,
				right_sum_gradients,
				right_sum_hessians,
			});
		}
	}
	best_split_for_feature
}

/// Choose the best discrete split for this feature.
fn choose_best_split_for_discrete_feature(
	feature_index: usize,
	binning_instructions: &BinningInstruction,
	bin_stats_for_feature: &[BinStatsEntry],
	n_examples_parent: usize,
	sum_gradients_parent: f64,
	sum_hessians_parent: f64,
	train_options: &TrainOptions,
) -> Option<ChooseBestSplitForFeatureOutput> {
	let mut best_split_for_feature: Option<ChooseBestSplitForFeatureOutput> = None;
	let l2_regularization = train_options.l2_regularization
		+ train_options.supplemental_l2_regularization_for_discrete_splits;
	let negative_loss_for_parent_node =
		compute_negative_loss(sum_gradients_parent, sum_hessians_parent, l2_regularization);
	let mut left_approximate_n_examples = 0;
	let mut left_sum_gradients = 0.0;
	let mut left_sum_hessians = 0.0;
	// Sort the bin stats using a scoring function.
	let smoothing_factor = train_options
		.smoothing_factor_for_discrete_bin_sorting
		.to_f64()
		.unwrap();
	let mut sorted_bin_stats_for_feature: Vec<(usize, &BinStatsEntry)> =
		bin_stats_for_feature.iter().enumerate().collect();
	sorted_bin_stats_for_feature.sort_by(|(_, a), (_, b)| {
		let score_a = a.sum_gradients / (a.sum_hessians + smoothing_factor);
		let score_b = b.sum_gradients / (b.sum_hessians + smoothing_factor);
		score_a.partial_cmp(&score_b).unwrap()
	});
	// For each bin, determine if splitting at that bin's value would produce a better split.
	let mut directions = vec![SplitDirection::Right; binning_instructions.n_bins()];
	for (bin_index, bin_stats_entry) in
		sorted_bin_stats_for_feature[0..bin_stats_for_feature.len() - 1].iter()
	{
		*directions.get_mut(*bin_index).unwrap() = SplitDirection::Left;
		// Approximate the number of examples that would be sent to the left child by assuming the fraction of examples is equal to the fraction of the sum of hessians.
		left_approximate_n_examples += (bin_stats_entry.sum_hessians
			* n_examples_parent.to_f64().unwrap()
			/ sum_hessians_parent)
			.round()
			.to_usize()
			.unwrap();
		left_sum_gradients += bin_stats_entry.sum_gradients;
		left_sum_hessians += bin_stats_entry.sum_hessians;
		// Above we approximate the number of examples based on the sum of hessians. It is possible this approximation is off by enough that left_approximate_n_examples exceeds n_examples_parent. If this happens, we must not consider any further bins as the split value by exiting the loop.
		let right_approximate_n_examples =
			match n_examples_parent.checked_sub(left_approximate_n_examples) {
				Some(right_n_examples) => right_n_examples,
				None => break,
			};
		// Compute the sum of gradients and hessians for the examples that would be sent to the right by this split. To make this fast, subtract the values for the left child from the parent.
		let right_sum_gradients = sum_gradients_parent - left_sum_gradients;
		let right_sum_hessians = sum_hessians_parent - left_sum_hessians;
		// Check if fewer than `min_examples_per_node` would be sent to the left child by this split.
		if left_approximate_n_examples < train_options.min_examples_per_node {
			continue;
		}
		// Check if fewer than `min_examples_per_node` would be sent to the right child by this split. If true, then splitting by the thresholds for all subsequent bins will also fail, so we can exit the loop.
		if right_approximate_n_examples < train_options.min_examples_per_node {
			break;
		}
		// Check if the sum of hessians for examples that would be sent to the left child by this split falls below `min_sum_hessians_per_node`.
		if left_sum_hessians < train_options.min_sum_hessians_per_node.to_f64().unwrap() {
			continue;
		}
		// Check if the sum of hessians for examples that would be sent to the right child by this split falls below `min_sum_hessians_per_node`. If true, then splitting by the thresholds for all subsequent bins will also fail, so we can exit the loop. This is true because hessians are always positive.
		if right_sum_hessians < train_options.min_sum_hessians_per_node.to_f64().unwrap() {
			break;
		}
		// Compute the gain for this candidate split.
		let gain = compute_gain(
			left_sum_gradients,
			left_sum_hessians,
			right_sum_gradients,
			right_sum_hessians,
			negative_loss_for_parent_node,
			l2_regularization,
		);
		// If this split has a higher gain or if there is no existing best split, then use this split.
		if best_split_for_feature
			.as_ref()
			.map(|best_split_for_feature| gain > best_split_for_feature.gain)
			.unwrap_or(true)
		{
			let split = TrainBranchSplit::Discrete(TrainBranchSplitDiscrete {
				feature_index,
				directions: directions.clone(),
			});
			best_split_for_feature = Some(ChooseBestSplitForFeatureOutput {
				gain,
				split,
				left_approximate_n_examples,
				left_sum_gradients,
				left_sum_hessians,
				right_approximate_n_examples,
				right_sum_gradients,
				right_sum_hessians,
			});
		}
	}
	best_split_for_feature
}

/// Compute the gain for a candidate split.
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
			let num_threads = rayon::current_num_threads();
			let chunk_size = (smaller_child_examples_index.len() + num_threads - 1) / num_threads;
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

/// Choose the splits for the left and right children with the highest gain.
fn choose_splits_with_highest_gain(
	children_best_splits_for_features: Vec<(
		Option<ChooseBestSplitForFeatureOutput>,
		Option<ChooseBestSplitForFeatureOutput>,
	)>,
) -> (
	Option<ChooseBestSplitForFeatureOutput>,
	Option<ChooseBestSplitForFeatureOutput>,
) {
	children_best_splits_for_features.into_iter().fold(
		(None, None),
		|(current_left, current_right), (candidate_left, candidate_right)| {
			(
				choose_split_with_highest_gain(current_left, candidate_left),
				choose_split_with_highest_gain(current_right, candidate_right),
			)
		},
	)
}

fn choose_split_with_highest_gain(
	current: Option<ChooseBestSplitForFeatureOutput>,
	candidate: Option<ChooseBestSplitForFeatureOutput>,
) -> Option<ChooseBestSplitForFeatureOutput> {
	match (current, candidate) {
		(None, None) => None,
		(current, None) => current,
		(None, candidate) => candidate,
		(Some(current), Some(candidate)) => {
			if candidate.gain > current.gain {
				Some(candidate)
			} else {
				Some(current)
			}
		}
	}
}
