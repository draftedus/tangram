use super::{
	bin::{BinnedFeatures, BinnedFeaturesColumn},
	TrainBranchSplit, TrainBranchSplitContinuous, TrainBranchSplitDiscrete,
};
use crate::SplitDirection;
use num_traits::ToPrimitive;
use rayon::prelude::*;

const MIN_EXAMPLES_TO_PARALLELIZE: usize = 100_000;

/// This function returns the `examples_index_range`s for the left and right nodes and rearranges the `examples_index` so that the example indexes in the first returned range correspond to the examples sent by the split to the left node and the example indexes in the second returned range correspond to the examples sent by the split to the right node.
pub fn rearrange_examples_index(
	binned_features: &BinnedFeatures,
	split: &TrainBranchSplit,
	examples_index: &mut [usize],
	examples_index_left_buffer: &mut [usize],
	examples_index_right_buffer: &mut [usize],
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
	if examples_index.len() <= MIN_EXAMPLES_TO_PARALLELIZE {
		rearrange_examples_index_serial(binned_features, split, examples_index)
	} else {
		rearrange_examples_index_parallel(
			binned_features,
			split,
			examples_index,
			examples_index_left_buffer,
			examples_index_right_buffer,
		)
	}
}

/// Rearrange the examples index on a single thread.
fn rearrange_examples_index_serial(
	binned_features: &BinnedFeatures,
	split: &TrainBranchSplit,
	examples_index: &mut [usize],
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
	let mut left = 0;
	let mut right = examples_index.len();
	match &split {
		TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
			feature_index,
			bin_index,
			..
		}) => {
			let binned_feature = &binned_features.columns[*feature_index];
			match binned_feature {
				BinnedFeaturesColumn::U8(binned_feature) => unsafe {
					rearrange_examples_index_serial_continuous(
						&mut left,
						&mut right,
						*bin_index,
						examples_index,
						&binned_feature,
					)
				},
				BinnedFeaturesColumn::U16(binned_feature) => unsafe {
					rearrange_examples_index_serial_continuous(
						&mut left,
						&mut right,
						*bin_index,
						examples_index,
						&binned_feature,
					)
				},
			}
		}
		TrainBranchSplit::Discrete(TrainBranchSplitDiscrete {
			feature_index,
			directions,
			..
		}) => {
			let binned_feature = &binned_features.columns[*feature_index];
			match binned_feature {
				BinnedFeaturesColumn::U8(binned_feature) => unsafe {
					rearrange_examples_index_serial_discrete(
						&mut left,
						&mut right,
						directions,
						examples_index,
						&binned_feature,
					)
				},
				BinnedFeaturesColumn::U16(binned_feature) => unsafe {
					rearrange_examples_index_serial_discrete(
						&mut left,
						&mut right,
						directions,
						examples_index,
						&binned_feature,
					)
				},
			}
		}
	}
	(0..left, left..examples_index.len())
}

unsafe fn rearrange_examples_index_serial_continuous<T>(
	left: &mut usize,
	right: &mut usize,
	bin_index: usize,
	examples_index: &mut [usize],
	binned_feature: &[T],
) where
	T: ToPrimitive,
{
	while left < right {
		let example_index = *examples_index.get_unchecked(*left);
		let binned_feature_value = binned_feature
			.get_unchecked(example_index)
			.to_usize()
			.unwrap();
		if binned_feature_value <= bin_index {
			*left += 1;
		} else {
			*right -= 1;
			std::ptr::swap_nonoverlapping(
				examples_index.as_mut_ptr().add(*left),
				examples_index.as_mut_ptr().add(*right),
				1,
			);
		}
	}
}

unsafe fn rearrange_examples_index_serial_discrete<T>(
	left: &mut usize,
	right: &mut usize,
	directions: &[SplitDirection],
	examples_index: &mut [usize],
	binned_feature: &[T],
) where
	T: ToPrimitive,
{
	while left < right {
		let example_index = *examples_index.get_unchecked(*left);
		let binned_feature_value = binned_feature
			.get_unchecked(example_index)
			.to_usize()
			.unwrap();
		if *directions.get_unchecked(binned_feature_value) == SplitDirection::Left {
			*left += 1;
		} else {
			*right -= 1;
			std::ptr::swap_nonoverlapping(
				examples_index.as_mut_ptr().add(*left),
				examples_index.as_mut_ptr().add(*right),
				1,
			);
		}
	}
}

/// Rearrange the examples index with multiple threads. This is done by segmenting the `examples_index` into chunks and writing the indexes of the examples that will be sent left and right by the split to chunks of the temporary buffers `examples_index_left_buffer` and `examples_index_right_buffer`. Then, the parts of each chunk that were written to are copied to the real examples index.
fn rearrange_examples_index_parallel(
	binned_features: &BinnedFeatures,
	split: &TrainBranchSplit,
	examples_index: &mut [usize],
	examples_index_left_buffer: &mut [usize],
	examples_index_right_buffer: &mut [usize],
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
	let chunk_size = examples_index.len() / rayon::current_num_threads();
	let counts: Vec<(usize, usize)> = (
		examples_index.par_chunks_mut(chunk_size),
		examples_index_left_buffer.par_chunks_mut(chunk_size),
		examples_index_right_buffer.par_chunks_mut(chunk_size),
	)
		.into_par_iter()
		.map(
			|(examples_index, examples_index_left_buffer, examples_index_right_buffer)| {
				let mut n_left = 0;
				let mut n_right = 0;
				match &split {
					TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
						feature_index,
						bin_index,
						..
					}) => {
						let binned_feature = &binned_features.columns[*feature_index];
						match binned_feature {
							BinnedFeaturesColumn::U8(binned_feature) => unsafe {
								rearrange_examples_index_parallel_step_one_continuous(
									&mut n_left,
									&mut n_right,
									*bin_index,
									examples_index,
									examples_index_left_buffer,
									examples_index_right_buffer,
									&binned_feature,
								)
							},
							BinnedFeaturesColumn::U16(binned_feature) => unsafe {
								rearrange_examples_index_parallel_step_one_continuous(
									&mut n_left,
									&mut n_right,
									*bin_index,
									examples_index,
									examples_index_left_buffer,
									examples_index_right_buffer,
									&binned_feature,
								)
							},
						}
					}
					TrainBranchSplit::Discrete(TrainBranchSplitDiscrete {
						feature_index,
						directions,
						..
					}) => {
						let binned_feature = &binned_features.columns[*feature_index];
						match binned_feature {
							BinnedFeaturesColumn::U8(binned_feature) => unsafe {
								rearrange_examples_index_parallel_step_one_discrete(
									&mut n_left,
									&mut n_right,
									directions,
									examples_index,
									examples_index_left_buffer,
									examples_index_right_buffer,
									&binned_feature,
								)
							},
							BinnedFeaturesColumn::U16(binned_feature) => unsafe {
								rearrange_examples_index_parallel_step_one_discrete(
									&mut n_left,
									&mut n_right,
									directions,
									examples_index,
									examples_index_left_buffer,
									examples_index_right_buffer,
									&binned_feature,
								)
							},
						}
					}
				}
				(n_left, n_right)
			},
		)
		.collect();
	let mut left_starting_indexes: Vec<(usize, usize)> = Vec::with_capacity(counts.len());
	let mut left_starting_index = 0;
	for (n_left, _) in counts.iter() {
		left_starting_indexes.push((left_starting_index, *n_left));
		left_starting_index += n_left;
	}
	let mut right_starting_indexes: Vec<(usize, usize)> = Vec::with_capacity(counts.len());
	let mut right_starting_index = left_starting_index;
	for (_, n_right) in counts.iter() {
		right_starting_indexes.push((right_starting_index, *n_right));
		right_starting_index += n_right;
	}
	(
		left_starting_indexes,
		right_starting_indexes,
		examples_index_left_buffer.par_chunks_mut(chunk_size),
		examples_index_right_buffer.par_chunks_mut(chunk_size),
	)
		.into_par_iter()
		.for_each(
			|(
				(left_starting_index, n_left),
				(right_starting_index, n_right),
				examples_index_left_buffer,
				examples_index_right_buffer,
			)| {
				let examples_index_slice =
					&examples_index[left_starting_index..left_starting_index + n_left];
				let examples_index_slice = unsafe {
					std::slice::from_raw_parts_mut(
						examples_index_slice.as_ptr() as *mut usize,
						examples_index_slice.len(),
					)
				};
				examples_index_slice.copy_from_slice(&examples_index_left_buffer[0..n_left]);
				let examples_index_slice =
					&examples_index[right_starting_index..right_starting_index + n_right];
				let examples_index_slice = unsafe {
					std::slice::from_raw_parts_mut(
						examples_index_slice.as_ptr() as *mut usize,
						examples_index_slice.len(),
					)
				};
				examples_index_slice.copy_from_slice(&examples_index_right_buffer[0..n_right]);
			},
		);
	(
		0..left_starting_index,
		left_starting_index..examples_index.len(),
	)
}

unsafe fn rearrange_examples_index_parallel_step_one_continuous<T>(
	n_left: &mut usize,
	n_right: &mut usize,
	bin_index: usize,
	examples_index: &mut [usize],
	examples_index_left_buffer: &mut [usize],
	examples_index_right_buffer: &mut [usize],
	binned_feature: &[T],
) where
	T: ToPrimitive,
{
	for example_index in examples_index {
		let binned_feature_value = binned_feature
			.get_unchecked(*example_index)
			.to_usize()
			.unwrap();
		if binned_feature_value <= bin_index {
			*examples_index_left_buffer.get_unchecked_mut(*n_left) = *example_index;
			*n_left += 1;
		} else {
			*examples_index_right_buffer.get_unchecked_mut(*n_right) = *example_index;
			*n_right += 1;
		}
	}
}

unsafe fn rearrange_examples_index_parallel_step_one_discrete<T>(
	n_left: &mut usize,
	n_right: &mut usize,
	directions: &[SplitDirection],
	examples_index: &mut [usize],
	examples_index_left_buffer: &mut [usize],
	examples_index_right_buffer: &mut [usize],
	binned_feature: &[T],
) where
	T: ToPrimitive,
{
	for example_index in examples_index {
		let binned_feature_value = binned_feature
			.get_unchecked(*example_index)
			.to_usize()
			.unwrap();
		if *directions.get_unchecked(binned_feature_value) == SplitDirection::Left {
			*examples_index_left_buffer.get_unchecked_mut(*n_left) = *example_index;
			*n_left += 1;
		} else {
			*examples_index_right_buffer.get_unchecked_mut(*n_right) = *example_index;
			*n_right += 1;
		}
	}
}
