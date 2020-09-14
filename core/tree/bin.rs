use itertools::{izip, Itertools};
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use tangram_dataframe::*;
use tangram_finite::Finite;

#[derive(Clone, Debug)]
pub enum BinInfo {
	Number { thresholds: Vec<f32> },
	Enum { n_options: u8 },
}

impl BinInfo {
	/**
	Returns the number of valid bins. The total number of bins is the number of valid bins + 1 bin reserved for missing values.
	## Number Bins
	Numeric features have n valid bins equal to the number of thresholds + 1.
	### Example
	Given 3 thresholds: `[0.5, 1.5, 2]`
	There are 4 valid bins:
	* (-infinity, 0.5]
	* (0.5, 1.5]
	* (1.5, 2]
	* (2, infinity)
	### Enum Bins
	Enum features have n valid bins equal to the number of enum variants.
	*/
	pub fn n_valid_bins(&self) -> u8 {
		match self {
			Self::Number { thresholds } => (thresholds.len() + 1).to_u8().unwrap(),
			Self::Enum { n_options } => *n_options,
		}
	}
}

/// ComputeBinInfoOptions specifies how to compute bins for a given column.
pub struct ComputeBinInfoOptions {
	/// The maximum number of bins to use for the column. Used to determine the number of bins for numeric columns because enum columns need n_valid_bins equal to the number of enum variants.
	pub max_valid_bins: u8,
	/// The maximum number of samples to use in order to estimate bin thresholds. This setting is used exclusively for numeric columns. In order to find bin thresholds, we need to sort values and find the threshold cutoffs. To speed up the computation, instead of sorting all of the values in the column, we choose to sort a smaller subset to get an estimate of the quantile threshold cutoffs.
	pub max_number_column_examples_for_bin_info: usize,
}

pub fn compute_bin_info(features: &DataFrameView, options: &ComputeBinInfoOptions) -> Vec<BinInfo> {
	features
		.columns
		.iter()
		.map(|column| compute_bin_info_for_column(column, &options))
		.collect()
}

/// Computes the bin info given a column.
pub fn compute_bin_info_for_column(
	column: &ColumnView,
	options: &ComputeBinInfoOptions,
) -> BinInfo {
	match column {
		ColumnView::Number(column) => compute_bin_info_for_number_column(column, options),
		ColumnView::Enum(column) => BinInfo::Enum {
			n_options: column.options.len().to_u8().unwrap(),
		},
		_ => unreachable!(),
	}
}

/// Computes the quantile thresholds for a numeric column. Returns BinInfo with the numeric thresholds used to map the column values into their respective bins.
fn compute_bin_info_for_number_column(
	column: &NumberColumnView,
	options: &ComputeBinInfoOptions,
) -> BinInfo {
	// collect the values into a histogram
	let mut histogram: BTreeMap<Finite<f32>, usize> = BTreeMap::new();
	let mut histogram_values_count = 0;
	for value in &column.data[0..column
		.data
		.len()
		.min(options.max_number_column_examples_for_bin_info)]
	{
		if let Ok(value) = Finite::new(*value) {
			*histogram.entry(value).or_insert(0) += 1;
			histogram_values_count += 1;
		}
	}
	// if the number of unique values is less than max_valid_bins,
	// then create one bin per unique value value. Otherwise,
	// create bins at quantiles.
	let thresholds = if histogram.len() < options.max_valid_bins.to_usize().unwrap() {
		histogram
			.keys()
			.tuple_windows()
			.map(|(a, b)| (a.get() + b.get()) / 2.0)
			.collect()
	} else {
		compute_bin_thresholds_for_histogram(
			histogram,
			histogram_values_count,
			options.max_valid_bins,
		)
	};
	BinInfo::Number { thresholds }
}

/// Computes the bin thresholds given a histogram of numeric values.
/// Instead of storing and sorting all values as an array, we collect values into a histogram which reduces the memory needed to compute thresholds for columns with many duplicate values.
fn compute_bin_thresholds_for_histogram(
	histogram: BTreeMap<Finite<f32>, usize>,
	histogram_values_count: usize,
	max_valid_bins: u8,
) -> Vec<f32> {
	let total_values_count = histogram_values_count.to_f32().unwrap();
	let quantiles: Vec<f32> = (1..max_valid_bins.to_usize().unwrap())
		.map(|i| i.to_f32().unwrap() / max_valid_bins.to_f32().unwrap())
		.collect();
	let quantile_indexes: Vec<usize> = quantiles
		.iter()
		.map(|q| ((total_values_count - 1.0) * q).trunc().to_usize().unwrap())
		.collect();
	let quantile_fracts: Vec<f32> = quantiles
		.iter()
		.map(|q| ((total_values_count - 1.0) * q).fract())
		.collect();
	let mut quantiles: Vec<Option<f32>> = vec![None; quantiles.len()];
	let mut current_count: usize = 0;
	let mut iter = histogram.iter().peekable();
	while let Some((value, count)) = iter.next() {
		let value = value.get();
		current_count += count;
		let quantiles_iter = quantiles
			.iter_mut()
			.zip(quantile_indexes.iter().zip(quantile_fracts.iter()))
			.filter(|(q, (_, _))| q.is_none());
		for (quantile, (index, fract)) in quantiles_iter {
			match (current_count - 1).cmp(index) {
				Ordering::Equal => {
					if *fract > 0.0 {
						let next_value = iter.peek().unwrap().0.get();
						*quantile = Some(value * (1.0 - fract) + next_value * fract);
					} else {
						*quantile = Some(value);
					}
				}
				Ordering::Greater => *quantile = Some(value),
				Ordering::Less => {}
			}
		}
	}
	quantiles.into_iter().map(|q| q.unwrap()).collect()
}

/// Computes the binned_features given raw input features and bin_info.
pub fn compute_binned_features(
	features: &DataFrameView,
	bin_info: &[BinInfo],
	max_n_bins: usize,
	progress: &(dyn Fn() + Sync),
) -> (Array2<u8>, Array2<usize>) {
	let n_examples = features.nrows();
	let n_features = features.ncols();
	let mut binned_features: Array2<u8> =
		unsafe { Array::uninitialized((n_examples, n_features).f()) };
	let mut binned_features_stats: Array2<usize> = Array::zeros((max_n_bins, n_features).f());
	izip!(
		binned_features.axis_iter_mut(Axis(1)),
		binned_features_stats.axis_iter_mut(Axis(1)),
		&features.columns,
		bin_info,
	)
	.for_each(
		|(mut binned_features_column, mut binned_feature_stats, column, bin_info)| {
			match (column, bin_info) {
				(ColumnView::Number(column), BinInfo::Number { thresholds }) => {
					for (binned_feature_value, feature_value) in
						binned_features_column.iter_mut().zip(column.data)
					{
						*binned_feature_value = if feature_value.is_nan() {
							0
						} else {
							// use binary search to find the bin for the feature value
							thresholds
								.binary_search_by(|threshold| {
									threshold.partial_cmp(feature_value).unwrap()
								})
								// reserve bin 0 for invalid
								.unwrap_or_else(|bin| bin)
								.to_u8()
								.unwrap() + 1
						};
						binned_feature_stats[*binned_feature_value as usize] += 1;
						progress();
					}
				}
				(ColumnView::Enum(column), BinInfo::Enum { .. }) => {
					for (binned_feature_value, feature_value) in
						binned_features_column.iter_mut().zip(column.data)
					{
						*binned_feature_value = feature_value.to_u8().unwrap();
						binned_feature_stats[binned_feature_value.to_usize().unwrap()] += 1;
						progress();
					}
				}
				_ => unreachable!(),
			}
		},
	);
	(binned_features, binned_features_stats)
}

pub struct FilterBinnedFeaturesOptions {
	pub min_examples_split: usize,
}

/** Filters out features that would result in invalid splits because no split exists such that there are more than min_examples_split.
For number features, iterate through bins sequentially to see if there exists a threshold such that min_examples_split is not violated.
For categorical features, determine if there exists any partition of the feature such that min_examples_split is not violated.
*/
pub fn filter_binned_features(
	binned_features: ArrayView2<u8>,
	binned_features_stats: Array2<usize>,
	bin_info: &[BinInfo],
	options: FilterBinnedFeaturesOptions,
) -> Vec<bool> {
	let n_examples = binned_features.nrows();
	binned_features_stats
		.axis_iter(Axis(1))
		.zip(bin_info)
		.map(|(binned_feature_stats, bin_info)| {
			match bin_info {
				BinInfo::Number { thresholds } => {
					// must be contiguous subset greater than threshold
					let mut count_so_far = 0;
					for count in binned_feature_stats
						.slice(s![0..thresholds.len() + 1])
						.iter()
					{
						count_so_far += count;
						if count_so_far >= options.min_examples_split
							&& n_examples - count_so_far >= options.min_examples_split
						{
							return true;
						}
					}
					false
				}
				BinInfo::Enum { n_options } => {
					// any subset is valid
					// sort the bins by size, if there is no partition where the left and right are above the threshold, don't include the feature
					let mut b = binned_feature_stats
						.slice(s![0..(*n_options as usize)])
						.to_vec();
					b.sort();
					let mut count_so_far = 0;
					for count in b.iter() {
						count_so_far += count;
						if count_so_far >= options.min_examples_split
							&& n_examples - count_so_far >= options.min_examples_split
						{
							return true;
						}
					}
					false
				}
			}
		})
		.collect::<Vec<bool>>()
}
