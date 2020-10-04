use super::binning::{BinnedFeatures, BinnedFeaturesColumn, BinningInstructions};
use itertools::izip;
use num_traits::ToPrimitive;
use rayon::prelude::*;
use tangram_thread_pool::pzip;

#[derive(Clone)]
pub struct BinStats(pub Vec<Vec<BinStatsEntry>>);

#[derive(Clone, Default)]
pub struct BinStatsEntry {
	pub sum_gradients: f64,
	pub sum_hessians: f64,
}

/// This value controls how far ahead in the `examples_index` the `compute_bin_stats_*` functions should prefetch binned_features to be used in subsequent iterations.
#[cfg(target_arch = "x86_64")]
const PREFETCH_OFFSET: usize = 64;

/// This value controls how many times to unroll the loop in `compute_bin_stats_for_feature_root`.
const ROOT_UNROLL: usize = 16;

/// This value controls how many times to unroll the loop in `compute_bin_stats_for_feature_not_root`.
const NOT_ROOT_UNROLL: usize = 4;

pub fn compute_bin_stats_for_feature_root(
	bin_stats_for_feature: &mut [BinStatsEntry],
	binned_feature: &BinnedFeaturesColumn,
	// (n_examples)
	gradients: &[f32],
	// (n_examples)
	hessians: &[f32],
	// hessians are constant in least squares loss, so we don't have to waste time updating them
	hessians_are_constant: bool,
) {
	for entry in bin_stats_for_feature.iter_mut() {
		*entry = BinStatsEntry {
			sum_gradients: 0.0,
			sum_hessians: 0.0,
		};
	}
	if hessians_are_constant {
		match binned_feature {
			BinnedFeaturesColumn::U8(binned_feature_values) => unsafe {
				compute_bin_stats_for_feature_root_no_hessians(
					gradients,
					binned_feature_values,
					bin_stats_for_feature,
				)
			},
			BinnedFeaturesColumn::U16(binned_feature_values) => unsafe {
				compute_bin_stats_for_feature_root_no_hessians(
					gradients,
					binned_feature_values,
					bin_stats_for_feature,
				)
			},
		}
	} else {
		match binned_feature {
			BinnedFeaturesColumn::U8(binned_feature_values) => unsafe {
				compute_bin_stats_for_feature_root_yes_hessians(
					gradients,
					hessians,
					binned_feature_values,
					bin_stats_for_feature,
				)
			},
			BinnedFeaturesColumn::U16(binned_feature_values) => unsafe {
				compute_bin_stats_for_feature_root_yes_hessians(
					gradients,
					hessians,
					binned_feature_values,
					bin_stats_for_feature,
				)
			},
		}
	}
}

pub fn compute_bin_stats_for_feature_not_root(
	smaller_child_bin_stats_for_feature: &mut [BinStatsEntry],
	smaller_child_examples_index: &[i32],
	binned_features_column: &BinnedFeaturesColumn,
	ordered_gradients: &[f32],
	ordered_hessians: &[f32],
	hessians_are_constant: bool,
) {
	for entry in smaller_child_bin_stats_for_feature.iter_mut() {
		*entry = BinStatsEntry {
			sum_gradients: 0.0,
			sum_hessians: 0.0,
		};
	}
	if hessians_are_constant {
		match binned_features_column {
			BinnedFeaturesColumn::U8(binned_feature_values) => unsafe {
				compute_bin_stats_for_feature_not_root_no_hessians(
					ordered_gradients,
					binned_feature_values.as_slice(),
					smaller_child_bin_stats_for_feature,
					smaller_child_examples_index,
				)
			},
			BinnedFeaturesColumn::U16(binned_feature_values) => unsafe {
				compute_bin_stats_for_feature_not_root_no_hessians(
					ordered_gradients,
					binned_feature_values.as_slice(),
					smaller_child_bin_stats_for_feature,
					smaller_child_examples_index,
				)
			},
		}
	} else {
		match binned_features_column {
			BinnedFeaturesColumn::U8(binned_feature_values) => unsafe {
				compute_bin_stats_for_feature_not_root_yes_hessians(
					ordered_gradients,
					ordered_hessians,
					binned_feature_values.as_slice(),
					smaller_child_bin_stats_for_feature,
					smaller_child_examples_index,
				)
			},
			BinnedFeaturesColumn::U16(binned_feature_values) => unsafe {
				compute_bin_stats_for_feature_not_root_yes_hessians(
					ordered_gradients,
					ordered_hessians,
					binned_feature_values.as_slice(),
					smaller_child_bin_stats_for_feature,
					smaller_child_examples_index,
				)
			},
		}
	}
}

pub fn compute_bin_stats_for_feature_not_root_subtraction(
	smaller_child_bin_stats_for_feature: &[BinStatsEntry],
	larger_child_bin_stats_for_feature: &mut [BinStatsEntry],
) {
	for (smaller_child_bin_stats, larger_child_bin_stats) in izip!(
		smaller_child_bin_stats_for_feature,
		larger_child_bin_stats_for_feature,
	) {
		larger_child_bin_stats.sum_gradients -= smaller_child_bin_stats.sum_gradients;
		larger_child_bin_stats.sum_hessians -= smaller_child_bin_stats.sum_hessians;
	}
}

unsafe fn compute_bin_stats_for_feature_root_no_hessians<T>(
	gradients: &[f32],
	binned_feature_values: &[T],
	bin_stats_for_feature: &mut [BinStatsEntry],
) where
	T: ToPrimitive,
{
	let unroll = ROOT_UNROLL;
	let len = gradients.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			let ordered_gradient = *gradients.get_unchecked(i);
			let bin_index = binned_feature_values.get_unchecked(i).to_usize().unwrap();
			let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
			bin_stats.sum_gradients += ordered_gradient as f64;
			bin_stats.sum_hessians += 1.0;
		}
	}
	for i in (len / unroll) * unroll..len {
		let ordered_gradient = *gradients.get_unchecked(i);
		let bin_index = binned_feature_values.get_unchecked(i).to_usize().unwrap();
		let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
		bin_stats.sum_gradients += ordered_gradient as f64;
		bin_stats.sum_hessians += 1.0;
	}
}

pub unsafe fn compute_bin_stats_for_feature_root_yes_hessians<T>(
	gradients: &[f32],
	hessians: &[f32],
	binned_feature_values: &[T],
	bin_stats_for_feature: &mut [BinStatsEntry],
) where
	T: ToPrimitive,
{
	let unroll = ROOT_UNROLL;
	let len = gradients.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			let ordered_gradient = *gradients.get_unchecked(i);
			let ordered_hessian = *hessians.get_unchecked(i);
			let bin_index = binned_feature_values.get_unchecked(i).to_usize().unwrap();
			let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
			bin_stats.sum_gradients += ordered_gradient as f64;
			bin_stats.sum_hessians += ordered_hessian as f64;
		}
	}
	for i in (len / unroll) * unroll..len {
		let ordered_gradient = *gradients.get_unchecked(i);
		let ordered_hessian = *hessians.get_unchecked(i);
		let bin_index = binned_feature_values.get_unchecked(i).to_usize().unwrap();
		let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
		bin_stats.sum_gradients += ordered_gradient as f64;
		bin_stats.sum_hessians += ordered_hessian as f64;
	}
}

unsafe fn compute_bin_stats_for_feature_not_root_no_hessians<T>(
	ordered_gradients: &[f32],
	binned_feature_values: &[T],
	bin_stats_for_feature: &mut [BinStatsEntry],
	examples_index: &[i32],
) where
	T: ToPrimitive,
{
	let unroll = NOT_ROOT_UNROLL;
	let len = examples_index.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			#[cfg(target_arch = "x86_64")]
			{
				let prefetch_index = examples_index
					.get_unchecked(i + PREFETCH_OFFSET)
					.to_usize()
					.unwrap();
				let prefetch_ptr = binned_feature_values.as_ptr().add(prefetch_index) as *const i8;
				core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
			}
			let ordered_gradient = *ordered_gradients.get_unchecked(i);
			let example_index = examples_index.get_unchecked(i).to_usize().unwrap();
			let bin_index = binned_feature_values
				.get_unchecked(example_index)
				.to_usize()
				.unwrap();
			let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
			bin_stats.sum_gradients += ordered_gradient as f64;
			bin_stats.sum_hessians += 1.0;
		}
	}
	for i in (len / unroll) * unroll..len {
		let ordered_gradient = *ordered_gradients.get_unchecked(i);
		let example_index = examples_index.get_unchecked(i).to_usize().unwrap();
		let bin_index = binned_feature_values
			.get_unchecked(example_index)
			.to_usize()
			.unwrap();
		let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
		bin_stats.sum_gradients += ordered_gradient as f64;
		bin_stats.sum_hessians += 1.0;
	}
}

pub unsafe fn compute_bin_stats_for_feature_not_root_yes_hessians<T>(
	ordered_gradients: &[f32],
	ordered_hessians: &[f32],
	binned_feature_values: &[T],
	bin_stats_for_feature: &mut [BinStatsEntry],
	examples_index: &[i32],
) where
	T: ToPrimitive,
{
	let unroll = NOT_ROOT_UNROLL;
	let len = examples_index.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			#[cfg(target_arch = "x86_64")]
			{
				let prefetch_index = examples_index
					.get_unchecked(i + PREFETCH_OFFSET)
					.to_usize()
					.unwrap();
				let prefetch_ptr = binned_feature_values.as_ptr().add(prefetch_index) as *const i8;
				core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
			}
			let ordered_gradient = *ordered_gradients.get_unchecked(i);
			let ordered_hessian = *ordered_hessians.get_unchecked(i);
			let example_index = examples_index.get_unchecked(i).to_usize().unwrap();
			let bin_index = binned_feature_values
				.get_unchecked(example_index)
				.to_usize()
				.unwrap();
			let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
			bin_stats.sum_gradients += ordered_gradient as f64;
			bin_stats.sum_hessians += ordered_hessian as f64;
		}
	}
	for i in (len / unroll) * unroll..len {
		let ordered_gradient = *ordered_gradients.get_unchecked(i);
		let ordered_hessian = *ordered_hessians.get_unchecked(i);
		let example_index = examples_index.get_unchecked(i).to_usize().unwrap();
		let bin_index = binned_feature_values
			.get_unchecked(example_index)
			.to_usize()
			.unwrap();
		let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
		bin_stats.sum_gradients += ordered_gradient as f64;
		bin_stats.sum_hessians += ordered_hessian as f64;
	}
}
