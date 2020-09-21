use super::{
	bin::BinnedFeaturesColumn, TrainBranchSplit, TrainBranchSplitContinuous,
	TrainBranchSplitDiscrete,
};
use crate::SplitDirection;
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;

/**
Returns the examples_index_ranges for the left and right nodes
and rearranges the examples_index so that the example indexes
in the first returned range are contained by the left node
and the example indexes in the second returned range
are contained by the right node.
*/
pub fn rearrange_examples_index(
	binned_features: &[BinnedFeaturesColumn],
	split: &TrainBranchSplit,
	examples_index: &mut [usize],
	examples_index_left: &mut [usize],
	examples_index_right: &mut [usize],
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
	if examples_index.len() <= 1024 {
		rearrange_examples_index_serial(binned_features, split, examples_index)
	} else {
		rearrange_examples_index_parallel(
			binned_features,
			split,
			examples_index,
			examples_index_left,
			examples_index_right,
		)
	}
}

/// Rearrange the examples index on a single thread.
fn rearrange_examples_index_serial(
	binned_features: &[BinnedFeaturesColumn],
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
					let binned_feature = &binned_features[*feature_index];
					let feature_bin = match binned_feature {
						BinnedFeaturesColumn::U8(binned_feature) => {
							binned_feature[examples_index[left]].to_u8().unwrap()
						}
						BinnedFeaturesColumn::U16(binned_feature) => {
							binned_feature[examples_index[left]].to_u8().unwrap()
						}
					};
					if feature_bin <= *bin_index {
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
					let binned_feature = &binned_features[*feature_index];
					let feature_bin = match binned_feature {
						BinnedFeaturesColumn::U8(binned_feature) => {
							binned_feature[examples_index[left]].to_usize().unwrap()
						}
						BinnedFeaturesColumn::U16(binned_feature) => {
							binned_feature[examples_index[left]].to_usize().unwrap()
						}
					};
					*directions.get(feature_bin).unwrap()
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

/// Rearrange the examples index with multiple threads.
fn rearrange_examples_index_parallel(
	binned_features: &[BinnedFeaturesColumn],
	split: &TrainBranchSplit,
	examples_index: &mut [usize],
	examples_index_left: &mut [usize],
	examples_index_right: &mut [usize],
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
	let chunk_size = usize::max(examples_index.len() / 16, 1024);
	let counts: Vec<(usize, usize)> = izip!(
		ArrayViewMut1::from(&mut examples_index[..]).axis_chunks_iter(Axis(0), chunk_size),
		ArrayViewMut1::from(&mut examples_index_left[..]).axis_chunks_iter_mut(Axis(0), chunk_size),
		ArrayViewMut1::from(&mut examples_index_right[..])
			.axis_chunks_iter_mut(Axis(0), chunk_size),
	)
	.map(
		|(examples_index, mut examples_index_left, mut examples_index_right)| {
			// update left and right examples indexes and return n_left and n_right
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
							let binned_features = &binned_features[*feature_index];
							let feature_bin = match binned_features {
								BinnedFeaturesColumn::U8(binned_features) => {
									binned_features[*example_index].to_u8().unwrap()
								}
								BinnedFeaturesColumn::U16(binned_features) => {
									binned_features[*example_index].to_u8().unwrap()
								}
							};
							if feature_bin <= *bin_index {
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
							let binned_features = &binned_features[*feature_index];
							let feature_bin = match binned_features {
								BinnedFeaturesColumn::U8(binned_features) => {
									binned_features[*example_index].to_usize().unwrap()
								}
								BinnedFeaturesColumn::U16(binned_features) => {
									binned_features[*example_index].to_usize().unwrap()
								}
							};
							*directions.get(feature_bin).unwrap()
						}
					}
				};
				match direction {
					SplitDirection::Left => {
						examples_index_left[n_left] = *example_index;
						n_left += 1;
					}
					SplitDirection::Right => {
						examples_index_right[n_right] = *example_index;
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
	izip!(
		left_starting_indexes,
		right_starting_indexes,
		ArrayViewMut1::from(&mut examples_index_left[..]).axis_chunks_iter(Axis(0), chunk_size),
		ArrayViewMut1::from(&mut examples_index_right[..]).axis_chunks_iter(Axis(0), chunk_size),
	)
	.for_each(
		|(
			(left_starting_index, n_left),
			(right_starting_index, n_right),
			examples_index_left,
			examples_index_right,
		)| {
			let examples_index_slice =
				&examples_index[left_starting_index..left_starting_index + n_left];
			let examples_index_slice = unsafe {
				std::slice::from_raw_parts_mut(
					examples_index_slice.as_ptr() as *mut usize,
					examples_index_slice.len(),
				)
			};
			examples_index_slice
				.copy_from_slice(examples_index_left.slice(s![0..n_left]).to_slice().unwrap());
			let examples_index_slice =
				&examples_index[right_starting_index..right_starting_index + n_right];
			let examples_index_slice = unsafe {
				std::slice::from_raw_parts_mut(
					examples_index_slice.as_ptr() as *mut usize,
					examples_index_slice.len(),
				)
			};
			examples_index_slice.copy_from_slice(
				examples_index_right
					.slice(s![0..n_right])
					.to_slice()
					.unwrap(),
			);
		},
	);
	(
		0..left_starting_index,
		left_starting_index..examples_index.len(),
	)
}
