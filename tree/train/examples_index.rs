use super::{
	bin::{BinnedFeatures, BinnedFeaturesColumn},
	TrainBranchSplit, TrainBranchSplitContinuous, TrainBranchSplitDiscrete,
};
use crate::SplitDirection;
use num_traits::ToPrimitive;
use rayon::prelude::*;

const MIN_EXAMPLES_TO_PARALLELIZE: usize = 1024;

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
	let start = 0;
	let end = examples_index.len();
	let mut left = start;
	let mut right = end;
	let mut n_left = 0;
	while left < right {
		let direction = {
			match &split {
				TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
					feature_index,
					bin_index,
					..
				}) => {
					let binned_feature = &binned_features.columns[*feature_index];
					let binned_feature_value = match binned_feature {
						BinnedFeaturesColumn::U8(binned_feature) => {
							binned_feature[examples_index[left]].to_usize().unwrap()
						}
						BinnedFeaturesColumn::U16(binned_feature) => {
							binned_feature[examples_index[left]].to_usize().unwrap()
						}
					};
					if binned_feature_value <= *bin_index {
						SplitDirection::Left
					} else {
						SplitDirection::Right
					}
				}
				TrainBranchSplit::Discrete(TrainBranchSplitDiscrete {
					feature_index,
					directions,
					..
				}) => {
					let binned_feature = &binned_features.columns[*feature_index];
					let binned_feature_value = match binned_feature {
						BinnedFeaturesColumn::U8(binned_feature) => {
							binned_feature[examples_index[left]].to_usize().unwrap()
						}
						BinnedFeaturesColumn::U16(binned_feature) => {
							binned_feature[examples_index[left]].to_usize().unwrap()
						}
					};
					*directions.get(binned_feature_value).unwrap()
				}
			}
		};
		match direction {
			SplitDirection::Left => {
				left += 1;
				n_left += 1;
			}
			SplitDirection::Right => {
				right -= 1;
				examples_index.swap(left, right);
			}
		};
	}
	(start..n_left, n_left..end)
}

const MIN_CHUNK_SIZE: usize = 1024;

/// Rearrange the examples index with multiple threads. This is done by segmenting the `examples_index` into chunks and writing the indexes of the examples that will be sent left and right by the split to chunks of the temporary buffers `examples_index_left_buffer` and `examples_index_right_buffer`. Then, the parts of each chunk that were written to are copied to the real examples index.
fn rearrange_examples_index_parallel(
	binned_features: &BinnedFeatures,
	split: &TrainBranchSplit,
	examples_index: &mut [usize],
	examples_index_left_buffer: &mut [usize],
	examples_index_right_buffer: &mut [usize],
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
	let chunk_size = usize::max(
		examples_index.len() / rayon::current_num_threads(),
		MIN_CHUNK_SIZE,
	);
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
				for example_index in examples_index {
					let direction = {
						match &split {
							TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
								feature_index,
								bin_index,
								..
							}) => {
								let binned_feature = &binned_features.columns[*feature_index];
								let binned_feature_value = match binned_feature {
									BinnedFeaturesColumn::U8(binned_features) => {
										binned_features[*example_index].to_usize().unwrap()
									}
									BinnedFeaturesColumn::U16(binned_features) => {
										binned_features[*example_index].to_usize().unwrap()
									}
								};
								if binned_feature_value <= *bin_index {
									SplitDirection::Left
								} else {
									SplitDirection::Right
								}
							}
							TrainBranchSplit::Discrete(TrainBranchSplitDiscrete {
								feature_index,
								directions,
								..
							}) => {
								let binned_feature = &binned_features.columns[*feature_index];
								let binned_feature_value = match binned_feature {
									BinnedFeaturesColumn::U8(binned_features) => {
										binned_features[*example_index].to_usize().unwrap()
									}
									BinnedFeaturesColumn::U16(binned_features) => {
										binned_features[*example_index].to_usize().unwrap()
									}
								};
								*directions.get(binned_feature_value).unwrap()
							}
						}
					};
					match direction {
						SplitDirection::Left => {
							examples_index_left_buffer[n_left] = *example_index;
							n_left += 1;
						}
						SplitDirection::Right => {
							examples_index_right_buffer[n_right] = *example_index;
							n_right += 1;
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
