use super::super::bin::BinInfo;
use itertools::izip;
use ndarray::prelude::*;

static PREFETCH_OFFSET: usize = 64;
static ROOT_UNROLL: usize = 16;
static NOT_ROOT_UNROLL: usize = 4;

#[derive(Clone)]
pub struct BinStatsPool {
	pub items: Vec<BinStats>,
}

#[derive(Clone)]
pub struct BinStats {
	/// One bin info per feature
	pub bin_info: Vec<BinInfo>,
	/// (n_features)
	pub entries: Vec<[f64; 512]>,
}

impl BinStatsPool {
	pub fn new(size: usize, bin_info: &[BinInfo]) -> Self {
		let mut items = Vec::with_capacity(size);
		for _ in 0..size {
			items.push(BinStats::new(bin_info.to_owned()));
		}
		Self { items }
	}
	pub fn get(&mut self) -> BinStats {
		self.items.pop().unwrap()
	}
}

impl BinStats {
	pub fn new(bin_info: Vec<BinInfo>) -> Self {
		let entries = vec![[0.0; 512]; bin_info.len()];
		Self { bin_info, entries }
	}
}

pub fn compute_bin_stats_for_root_node(
	node_bin_stats: &mut BinStats,
	include_features: &[bool],
	// (n_examples, n_features) column major
	binned_features: ArrayView2<u8>,
	// (n_examples)
	gradients: &[f32],
	// (n_examples)
	hessians: &[f32],
	// hessians are constant in least squares loss, so we don't have to waste time updating them
	hessians_are_constant: bool,
) {
	izip!(
		&mut node_bin_stats.bin_info,
		&mut node_bin_stats.entries,
		binned_features.gencolumns(),
		include_features,
	)
	.for_each(
		|(bin_info_for_feature, bin_stats_for_feature, binned_feature_values, include_feature)| {
			if !include_feature {
				return;
			}
			for entry in
				&mut bin_stats_for_feature[0..bin_info_for_feature.n_valid_bins() as usize * 2]
			{
				*entry = 0.0;
			}
			if hessians_are_constant {
				unsafe {
					compute_bin_stats_for_feature_root_no_hessian(
						gradients,
						binned_feature_values.as_slice().unwrap(),
						bin_stats_for_feature,
					)
				}
			} else {
				unsafe {
					compute_bin_stats_for_feature_root(
						gradients,
						hessians,
						binned_feature_values.as_slice().unwrap(),
						bin_stats_for_feature,
					)
				};
			}
		},
	);
}

#[allow(clippy::collapsible_if)]
#[allow(clippy::too_many_arguments)]
pub fn compute_bin_stats_for_non_root_node(
	node_bin_stats: &mut BinStats,
	include_features: &[bool],
	// (n_examples)
	ordered_gradients: &mut [f32],
	// (n_examples)
	ordered_hessians: &mut [f32],
	// (n_examples, n_features) column major
	binned_features: ArrayView2<u8>,
	// (n_examples)
	gradients: &[f32],
	// (n_examples)
	hessians: &[f32],
	// hessians are constant in least squares loss, so we don't have to waste time updating them
	hessians_are_constant: bool,
	// these are the indexes of the examples at this node, length only equal to n_examples at the root node
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
	izip!(
		&mut node_bin_stats.bin_info,
		&mut node_bin_stats.entries,
		binned_features.gencolumns(),
		include_features
	)
	.for_each(
		|(bin_info_for_feature, bin_stats_for_feature, binned_feature_values, include_feature)| {
			if !include_feature {
				return;
			}
			for entry in
				&mut bin_stats_for_feature[0..bin_info_for_feature.n_valid_bins() as usize * 2]
			{
				*entry = 0.0;
			}
			if hessians_are_constant {
				unsafe {
					compute_bin_stats_for_feature_not_root_no_hessians(
						ordered_gradients,
						binned_feature_values.as_slice().unwrap(),
						bin_stats_for_feature,
						examples_index_for_node,
					)
				}
			} else {
				unsafe {
					compute_bin_stats_for_feature_not_root(
						ordered_gradients,
						ordered_hessians,
						binned_feature_values.as_slice().unwrap(),
						bin_stats_for_feature,
						examples_index_for_node,
					)
				}
			}
		},
	);
}

unsafe fn compute_bin_stats_for_feature_root_no_hessian(
	gradients: &[f32],
	binned_feature_values: &[u8],
	bin_stats_for_feature: &mut [f64],
) {
	let unroll = ROOT_UNROLL;
	let len = gradients.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			let ordered_gradient = *gradients.get_unchecked(i);
			let bin_index = *binned_feature_values.get_unchecked(i) as usize;
			let bin_index = bin_index << 1;
			*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
			*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += 1.0;
		}
	}
	for i in (len / unroll) * unroll..len {
		let ordered_gradient = *gradients.get_unchecked(i);
		let bin_index = *binned_feature_values.get_unchecked(i) as usize;
		let bin_index = bin_index << 1;
		*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
		*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += 1.0;
	}
}

pub unsafe fn compute_bin_stats_for_feature_root(
	gradients: &[f32],
	hessians: &[f32],
	binned_feature_values: &[u8],
	bin_stats_for_feature: &mut [f64],
) {
	let unroll = ROOT_UNROLL;
	let len = gradients.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			let ordered_gradient = *gradients.get_unchecked(i);
			let ordered_hessian = *hessians.get_unchecked(i);
			let bin_index = *binned_feature_values.get_unchecked(i) as usize;
			let bin_index = bin_index << 1;
			*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
			*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += ordered_hessian as f64;
		}
	}
	for i in (len / unroll) * unroll..len {
		let ordered_gradient = *gradients.get_unchecked(i);
		let ordered_hessian = *hessians.get_unchecked(i);
		let bin_index = *binned_feature_values.get_unchecked(i) as usize;
		let bin_index = bin_index << 1;
		*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
		*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += ordered_hessian as f64;
	}
}

unsafe fn compute_bin_stats_for_feature_not_root_no_hessians(
	ordered_gradients: &[f32],
	binned_feature_values: &[u8],
	bin_stats_for_feature: &mut [f64],
	examples_index: &[usize],
) {
	let unroll = NOT_ROOT_UNROLL;
	let len = examples_index.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			let prefetch_index = *examples_index.get_unchecked(i + PREFETCH_OFFSET);
			let prefetch_ptr = binned_feature_values.as_ptr().add(prefetch_index) as *const i8;
			core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
			let ordered_gradient = *ordered_gradients.get_unchecked(i);
			let example_index = *examples_index.get_unchecked(i);
			let bin_index = *binned_feature_values.get_unchecked(example_index) as usize;
			let bin_index = bin_index << 1;
			*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
			*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += 1.0;
		}
	}
	for i in (len / unroll) * unroll..len {
		let ordered_gradient = *ordered_gradients.get_unchecked(i);
		let example_index = *examples_index.get_unchecked(i);
		let bin_index = *binned_feature_values.get_unchecked(example_index) as usize;
		let bin_index = bin_index << 1;
		*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
		*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += 1.0;
	}
}

unsafe fn compute_bin_stats_for_feature_not_root(
	ordered_gradients: &[f32],
	ordered_hessians: &[f32],
	binned_feature_values: &[u8],
	bin_stats_for_feature: &mut [f64],
	examples_index: &[usize],
) {
	let unroll = NOT_ROOT_UNROLL;
	let len = examples_index.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			let prefetch_index = *examples_index.get_unchecked(i + PREFETCH_OFFSET);
			let prefetch_ptr = binned_feature_values.as_ptr().add(prefetch_index) as *const i8;
			core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
			let ordered_gradient = *ordered_gradients.get_unchecked(i);
			let ordered_hessian = *ordered_hessians.get_unchecked(i);
			let example_index = *examples_index.get_unchecked(i);
			let bin_index = *binned_feature_values.get_unchecked(example_index) as usize;
			let bin_index = bin_index << 1;
			*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
			*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += ordered_hessian as f64;
		}
	}
	for i in (len / unroll) * unroll..len {
		let ordered_gradient = *ordered_gradients.get_unchecked(i);
		let ordered_hessian = *ordered_hessians.get_unchecked(i);
		let example_index = *examples_index.get_unchecked(i);
		let bin_index = *binned_feature_values.get_unchecked(example_index) as usize;
		let bin_index = bin_index << 1;
		*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
		*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += ordered_hessian as f64;
	}
}

// Subtracts the bin_stats for a sibling from the parent.
// The subtraction method:
// 1. Compute the bin_stats for the child node with less examples.
// 2. Get the bin_stats for the child node with more examples by subtracting
//    sibling_node_bin_stats from step 1 from the parent_bin_stats.
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

// Subtracts the sibling_bin_stats from the parent_bin_stats
// for a single feature.
fn compute_bin_stats_subtraction_for_feature(
	// (n_bins)
	parent_bin_stats_for_feature: &mut [f64],
	// (n_bins)
	sibling_bin_stats_for_feature: &[f64],
) {
	let iter = parent_bin_stats_for_feature
		.iter_mut()
		.zip(sibling_bin_stats_for_feature);
	for (parent_bin_stats, sibling_bin_stats) in iter {
		*parent_bin_stats -= sibling_bin_stats;
	}
}
