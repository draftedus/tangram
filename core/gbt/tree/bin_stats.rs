use super::super::bin::BinInfo;
use ndarray::prelude::*;
use ndarray::Zip;
use num_traits::ToPrimitive;

#[derive(Clone)]
pub struct BinStatsPool {
	pub items: Vec<BinStats>,
}

#[derive(Clone)]
pub struct BinStats {
	/// One bin info per feature
	pub bin_info: Vec<BinInfo>,
	/// the outer vec has length n_features,
	/// the inner vec has length n_bins for that feature
	pub entries: Vec<Vec<BinStatsEntry>>,
}

#[derive(Clone, Copy, Debug)]
pub struct BinStatsEntry {
	pub count: usize,
	pub sum_gradients: f64,
	pub sum_hessians: f64,
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
		let entries = bin_info
			.iter()
			.map(|bin_info| {
				let n_valid_bins = bin_info.n_valid_bins() as usize;
				let n_bins = n_valid_bins + 1;
				let mut vec = Vec::with_capacity(n_bins);
				unsafe { vec.set_len(n_bins) };
				vec
			})
			.collect();
		Self { bin_info, entries }
	}
}

impl BinStatsEntry {
	pub fn new() -> Self {
		Self {
			sum_gradients: 0.0,
			sum_hessians: 0.0,
			count: 0,
		}
	}
}

pub fn compute_bin_stats_for_root_node(
	node_bin_stats: &mut BinStats,
	include_features: &[bool],
	// (n_examples, n_features) column major
	binned_features: ArrayView2<u8>,
	// (n_examples)
	gradients: ArrayView1<f32>,
	// (n_examples)
	hessians: ArrayView1<f32>,
	// hessians are constant in least squares loss, so we don't have to waste time updating them
	hessians_are_constant: bool,
) {
	Zip::from(&mut node_bin_stats.entries)
		.and(binned_features.gencolumns())
		.and(include_features)
		.par_apply(
			|bin_stats_for_feature, binned_feature_values, include_feature| {
				let mut bin_stats_for_feature: ArrayViewMut1<BinStatsEntry> =
					bin_stats_for_feature.into();
				bin_stats_for_feature.fill(BinStatsEntry::new());
				if !include_feature {
					return;
				}
				if hessians_are_constant {
					compute_bin_stats_for_feature_root_no_hessian(
						gradients,
						binned_feature_values,
						bin_stats_for_feature,
					)
				} else {
					compute_bin_stats_for_feature_root(
						gradients,
						hessians,
						binned_feature_values,
						bin_stats_for_feature,
					)
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
	mut ordered_gradients: ArrayViewMut1<f32>,
	// (n_examples)
	mut ordered_hessians: ArrayViewMut1<f32>,
	// (n_examples, n_features) column major
	binned_features: ArrayView2<u8>,
	// (n_examples)
	gradients: ArrayView1<f32>,
	// (n_examples)
	hessians: ArrayView1<f32>,
	// hessians are constant in least squares loss, so we don't have to waste time updating them
	hessians_are_constant: bool,
	// these are the indexes of the examples at this node, length only equal to n_examples at the root node
	examples_index_for_node: ArrayView1<usize>,
) {
	let n_examples_in_node = examples_index_for_node.len();
	if !hessians_are_constant {
		if n_examples_in_node <= 1024 {
			for i in 0..n_examples_in_node {
				ordered_gradients[i] = gradients[examples_index_for_node[i]];
				ordered_hessians[i] = hessians[examples_index_for_node[i]]
			}
		} else {
			Zip::indexed(ordered_gradients.slice_mut(s![0..n_examples_in_node]))
				.and(ordered_hessians.slice_mut(s![0..n_examples_in_node]))
				.par_apply(|i, ordered_gradient, ordered_hessian| {
					*ordered_gradient = gradients[examples_index_for_node[i]];
					*ordered_hessian = hessians[examples_index_for_node[i]];
				});
		}
	} else {
		if n_examples_in_node <= 1024 {
			for i in 0..n_examples_in_node {
				ordered_gradients[i] = gradients[examples_index_for_node[i]];
			}
		} else {
			Zip::indexed(ordered_gradients.slice_mut(s![0..n_examples_in_node])).par_apply(
				|i, ordered_gradient| {
					*ordered_gradient = gradients[examples_index_for_node[i]];
				},
			);
		}
	}
	Zip::from(&mut node_bin_stats.entries)
		.and(binned_features.gencolumns())
		.and(include_features)
		.par_apply(
			|bin_stats_for_feature, binned_feature_values, include_feature| {
				let mut bin_stats_for_feature: ArrayViewMut1<BinStatsEntry> =
					bin_stats_for_feature.into();
				bin_stats_for_feature.fill(BinStatsEntry::new());
				if !include_feature {
					return;
				}
				if hessians_are_constant {
					compute_bin_stats_for_feature_not_root_no_hessians(
						ordered_gradients.view(),
						binned_feature_values,
						bin_stats_for_feature,
						examples_index_for_node,
					)
				} else {
					compute_bin_stats_for_feature_not_root(
						ordered_gradients.view(),
						ordered_hessians.view(),
						binned_feature_values,
						bin_stats_for_feature,
						examples_index_for_node,
					)
				}
			},
		);
}

fn compute_bin_stats_for_feature_root_no_hessian(
	gradients: ArrayView1<f32>,
	binned_feature_values: ArrayView1<u8>,
	mut bin_stats_for_feature: ArrayViewMut1<BinStatsEntry>,
) {
	for i in 0..gradients.len() {
		let ordered_gradient = unsafe { *gradients.uget(i) };
		let bin_index = unsafe { *binned_feature_values.uget(i) as usize };
		let bin_stats = unsafe { bin_stats_for_feature.uget_mut(bin_index) };
		bin_stats.sum_gradients += ordered_gradient.to_f64().unwrap();
		bin_stats.sum_hessians += 1.0;
		bin_stats.count += 1;
	}
}

fn compute_bin_stats_for_feature_root(
	gradients: ArrayView1<f32>,
	hessians: ArrayView1<f32>,
	binned_feature_values: ArrayView1<u8>,
	mut bin_stats_for_feature: ArrayViewMut1<BinStatsEntry>,
) {
	for i in 0..gradients.len() {
		let ordered_gradient = unsafe { *gradients.uget(i) };
		let ordered_hessian = unsafe { *hessians.uget(i) };
		let bin_index = unsafe { *binned_feature_values.uget(i) as usize };
		let bin_stats = unsafe { bin_stats_for_feature.uget_mut(bin_index) };
		bin_stats.sum_gradients += ordered_gradient.to_f64().unwrap();
		bin_stats.sum_hessians += ordered_hessian.to_f64().unwrap();
		bin_stats.count += 1;
	}
}

fn compute_bin_stats_for_feature_not_root_no_hessians(
	ordered_gradients: ArrayView1<f32>,
	binned_feature_values: ArrayView1<u8>,
	mut bin_stats_for_feature: ArrayViewMut1<BinStatsEntry>,
	examples_index: ArrayView1<usize>,
) {
	let n_examples = examples_index.len();
	for i in 0..n_examples {
		let ordered_gradient = unsafe { *ordered_gradients.uget(i) };
		let bin_index = unsafe { *binned_feature_values.uget(*examples_index.uget(i)) as usize };
		let bin_stats = unsafe { bin_stats_for_feature.uget_mut(bin_index) };
		bin_stats.sum_gradients += ordered_gradient.to_f64().unwrap();
		bin_stats.sum_hessians += 1.0;
		bin_stats.count += 1;
	}
}

fn compute_bin_stats_for_feature_not_root(
	ordered_gradients: ArrayView1<f32>,
	ordered_hessians: ArrayView1<f32>,
	binned_feature_values: ArrayView1<u8>,
	mut bin_stats_for_feature: ArrayViewMut1<BinStatsEntry>,
	examples_index: ArrayView1<usize>,
) {
	let n_examples = examples_index.len();
	for i in 0..n_examples {
		let ordered_gradient = unsafe { *ordered_gradients.uget(i) };
		let ordered_hessian = unsafe { *ordered_hessians.uget(i) };
		let bin_index = unsafe { *binned_feature_values.uget(*examples_index.uget(i)) as usize };
		let bin_stats = unsafe { bin_stats_for_feature.uget_mut(bin_index) };
		bin_stats.sum_gradients += ordered_gradient.to_f64().unwrap();
		bin_stats.sum_hessians += ordered_hessian.to_f64().unwrap();
		bin_stats.count += 1;
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
	//(n_bins)
	parent_bin_stats_for_feature: &mut [BinStatsEntry],
	//(n_bins)
	sibling_bin_stats_for_feature: &[BinStatsEntry],
) {
	let iter = parent_bin_stats_for_feature
		.iter_mut()
		.zip(sibling_bin_stats_for_feature);
	for (parent_bin_stats, sibling_bin_stats) in iter {
		parent_bin_stats.sum_gradients -= sibling_bin_stats.sum_gradients;
		parent_bin_stats.sum_hessians -= sibling_bin_stats.sum_hessians;
		parent_bin_stats.count -= sibling_bin_stats.count;
	}
}
