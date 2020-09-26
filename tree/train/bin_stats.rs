use super::bin::{BinnedFeatures, BinnedFeaturesColumn, BinningInstructions};
use itertools::izip;
use num_traits::ToPrimitive;
use tangram_pool::{Pool, PoolGuard};

#[derive(Clone)]
pub struct BinStats {
	pub binning_instructions: Vec<BinningInstructions>,
	pub entries: Vec<Vec<BinStatsEntry>>,
}

#[derive(Clone)]
pub struct BinStatsEntry {
	pub sum_gradients: f64,
	pub sum_hessians: f64,
}

impl BinStats {
	pub fn new(binning_instructions: Vec<BinningInstructions>) -> Self {
		let entries = binning_instructions
			.iter()
			.map(|b| {
				vec![
					BinStatsEntry {
						sum_gradients: 0.0,
						sum_hessians: 0.0
					};
					b.n_bins()
				]
			})
			.collect();
		Self {
			binning_instructions,
			entries,
		}
	}
}

pub struct BinStatsPool {
	pool: Pool<BinStats>,
}

impl BinStatsPool {
	pub fn new(max_size: usize, binning_instructions: &[BinningInstructions]) -> Self {
		let binning_instructions = binning_instructions.to_owned();
		let pool = Pool::new(
			max_size,
			Box::new(move || BinStats::new(binning_instructions.clone())),
		);
		Self { pool }
	}
	pub fn get(&mut self) -> PoolGuard<BinStats> {
		self.pool.get().unwrap()
	}
}

/// This value controls how far ahead in the `examples_index` the `compute_bin_stats_*` functions should prefetch binned_features to be used in subsequent iterations.
#[cfg(target_arch = "x86_64")]
const PREFETCH_OFFSET: usize = 64;

/// This value controls how many times to unroll the loop in `compute_bin_stats_for_feature_root`.
const ROOT_UNROLL: usize = 16;

/// This value controls how many times to unroll the loop in `compute_bin_stats_for_feature_not_root`.
const NOT_ROOT_UNROLL: usize = 4;

pub fn compute_bin_stats_for_root_node(
	node_bin_stats: &mut BinStats,
	binned_features: &BinnedFeatures,
	// (n_examples)
	gradients: &[f32],
	// (n_examples)
	hessians: &[f32],
	// hessians are constant in least squares loss, so we don't have to waste time updating them
	hessians_are_constant: bool,
) {
	izip!(&mut node_bin_stats.entries, binned_features.columns.iter(),).for_each(
		|(bin_stats_for_feature, binned_feature_values)| {
			for entry in bin_stats_for_feature.iter_mut() {
				*entry = BinStatsEntry {
					sum_gradients: 0.0,
					sum_hessians: 0.0,
				};
			}
			if hessians_are_constant {
				unsafe {
					match binned_feature_values {
						BinnedFeaturesColumn::U8(binned_feature_values) => {
							compute_bin_stats_for_feature_root_no_hessian(
								gradients,
								binned_feature_values,
								bin_stats_for_feature,
							)
						}
						BinnedFeaturesColumn::U16(binned_feature_values) => {
							compute_bin_stats_for_feature_root_no_hessian(
								gradients,
								binned_feature_values,
								bin_stats_for_feature,
							)
						}
					}
				}
			} else {
				unsafe {
					match binned_feature_values {
						BinnedFeaturesColumn::U8(binned_feature_values) => {
							compute_bin_stats_for_feature_root(
								gradients,
								hessians,
								binned_feature_values,
								bin_stats_for_feature,
							)
						}
						BinnedFeaturesColumn::U16(binned_feature_values) => {
							compute_bin_stats_for_feature_root(
								gradients,
								hessians,
								binned_feature_values,
								bin_stats_for_feature,
							)
						}
					}
				};
			}
		},
	);
}

#[allow(clippy::collapsible_if)]
#[allow(clippy::too_many_arguments)]
pub fn compute_bin_stats_for_non_root_node(
	node_bin_stats: &mut BinStats,
	ordered_gradients: &mut [f32],
	ordered_hessians: &mut [f32],
	binned_features: &BinnedFeatures,
	gradients: &[f32],
	hessians: &[f32],
	hessians_are_constant: bool,
	examples_index_for_node: &[usize],
) {
	let n_examples_in_node = examples_index_for_node.len();
	if !hessians_are_constant {
		for i in 0..n_examples_in_node {
			ordered_gradients[i] = gradients[examples_index_for_node[i]];
			ordered_hessians[i] = hessians[examples_index_for_node[i]]
		}
	} else {
		for i in 0..n_examples_in_node {
			ordered_gradients[i] = gradients[examples_index_for_node[i]];
		}
	}
	izip!(&mut node_bin_stats.entries, binned_features.columns.iter(),).for_each(
		|(bin_stats_for_feature, binned_feature_values)| {
			for entry in bin_stats_for_feature.iter_mut() {
				*entry = BinStatsEntry {
					sum_gradients: 0.0,
					sum_hessians: 0.0,
				};
			}
			if hessians_are_constant {
				unsafe {
					match binned_feature_values {
						BinnedFeaturesColumn::U8(binned_feature_values) => {
							compute_bin_stats_for_feature_not_root_no_hessians(
								ordered_gradients,
								binned_feature_values.as_slice(),
								bin_stats_for_feature,
								examples_index_for_node,
							)
						}
						BinnedFeaturesColumn::U16(binned_feature_values) => {
							compute_bin_stats_for_feature_not_root_no_hessians(
								ordered_gradients,
								binned_feature_values.as_slice(),
								bin_stats_for_feature,
								examples_index_for_node,
							)
						}
					}
				}
			} else {
				unsafe {
					match binned_feature_values {
						BinnedFeaturesColumn::U8(binned_feature_values) => {
							compute_bin_stats_for_feature_not_root(
								ordered_gradients,
								ordered_hessians,
								binned_feature_values.as_slice(),
								bin_stats_for_feature,
								examples_index_for_node,
							)
						}
						BinnedFeaturesColumn::U16(binned_feature_values) => {
							compute_bin_stats_for_feature_not_root(
								ordered_gradients,
								ordered_hessians,
								binned_feature_values.as_slice(),
								bin_stats_for_feature,
								examples_index_for_node,
							)
						}
					}
				}
			}
		},
	);
}

unsafe fn compute_bin_stats_for_feature_root_no_hessian<T>(
	gradients: &[f32],
	binned_feature_values: &[T],
	bin_stats_for_feature: &mut [BinStatsEntry],
) where
	T: num_traits::cast::ToPrimitive,
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

pub unsafe fn compute_bin_stats_for_feature_root<T>(
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
	examples_index: &[usize],
) where
	T: num_traits::cast::ToPrimitive,
{
	let unroll = NOT_ROOT_UNROLL;
	let len = examples_index.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			#[cfg(target_arch = "x86_64")]
			{
				let prefetch_index = *examples_index.get_unchecked(i + PREFETCH_OFFSET);
				let prefetch_ptr = binned_feature_values.as_ptr().add(prefetch_index) as *const i8;
				core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
			}
			let ordered_gradient = *ordered_gradients.get_unchecked(i);
			let example_index = *examples_index.get_unchecked(i);
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
		let example_index = *examples_index.get_unchecked(i);
		let bin_index = binned_feature_values
			.get_unchecked(example_index)
			.to_usize()
			.unwrap();
		let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
		bin_stats.sum_gradients += ordered_gradient as f64;
		bin_stats.sum_hessians += 1.0;
	}
}

unsafe fn compute_bin_stats_for_feature_not_root<T>(
	ordered_gradients: &[f32],
	ordered_hessians: &[f32],
	binned_feature_values: &[T],
	bin_stats_for_feature: &mut [BinStatsEntry],
	examples_index: &[usize],
) where
	T: num_traits::cast::ToPrimitive,
{
	let unroll = NOT_ROOT_UNROLL;
	let len = examples_index.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			#[cfg(target_arch = "x86_64")]
			{
				let prefetch_index = *examples_index.get_unchecked(i + PREFETCH_OFFSET);
				let prefetch_ptr = binned_feature_values.as_ptr().add(prefetch_index) as *const i8;
				core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
			}
			let ordered_gradient = *ordered_gradients.get_unchecked(i);
			let ordered_hessian = *ordered_hessians.get_unchecked(i);
			let example_index = *examples_index.get_unchecked(i);
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
		let example_index = *examples_index.get_unchecked(i);
		let bin_index = binned_feature_values
			.get_unchecked(example_index)
			.to_usize()
			.unwrap();
		let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
		bin_stats.sum_gradients += ordered_gradient as f64;
		bin_stats.sum_hessians += ordered_hessian as f64;
	}
}

// Subtracts the bin_stats for a sibling from the parent.
// The subtraction method:
// 1. Compute the bin_stats for the child node with less examples.
// 2. Get the bin_stats for the child node with more examples by subtracting sibling_node_bin_stats from step 1 from the parent_bin_stats.
pub fn compute_bin_stats_subtraction(
	// (n_features, n_bins)
	parent_bin_stats: &mut BinStats,
	// (n_features, n_bins)
	sibling_bin_stats: &BinStats,
) {
	let iter = parent_bin_stats
		.entries
		.iter_mut()
		.zip(sibling_bin_stats.entries.iter());
	for (parent_bin_stats_for_feature, sibling_bin_stats_for_feature) in iter {
		compute_bin_stats_subtraction_for_feature(
			parent_bin_stats_for_feature,
			sibling_bin_stats_for_feature,
		);
	}
}

/// Subtracts the sibling_bin_stats from the parent_bin_stats for a single feature.
fn compute_bin_stats_subtraction_for_feature(
	// (n_bins)
	parent_bin_stats_for_feature: &mut [BinStatsEntry],
	// (n_bins)
	sibling_bin_stats_for_feature: &[BinStatsEntry],
) {
	let iter = parent_bin_stats_for_feature
		.iter_mut()
		.zip(sibling_bin_stats_for_feature);
	for (parent_bin_stats, sibling_bin_stats) in iter {
		parent_bin_stats.sum_gradients -= sibling_bin_stats.sum_gradients;
		parent_bin_stats.sum_hessians -= sibling_bin_stats.sum_hessians;
	}
}
