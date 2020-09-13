use super::dataset::*;
use crate::{metrics, util::finite::Finite};
use num_traits::ToPrimitive;
use std::{cmp::Ordering, collections::BTreeMap, num::NonZeroU64};

/// HistogramStats contain statistics computed using aggregated histograms of the original column. We use aggregated histogram statistics for computing quantiles on number columns. Instead of sorting `O(n_examples)`, we only need to sort `O(n_unique_values)`.
#[derive(Debug, PartialEq)]
pub enum HistogramStats {
	Unknown(UnknownHistogramStats),
	Text(TextHistogramStats),
	Number(NumberHistogramStats),
	Enum(EnumHistogramStats),
}

/// UnknownHistogramStats are empty.
#[derive(Debug, PartialEq)]
pub struct UnknownHistogramStats {}

/// TextHistogramStats are empty.
#[derive(Debug, PartialEq)]
pub struct TextHistogramStats {}

/// NumberHistogramStats contain statistics computed using aggregated histograms of the original column.
#[derive(Debug, PartialEq)]
pub struct NumberHistogramStats {
	pub mean: f32,
	pub variance: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
	pub max: f32,
	pub binned_histogram: Option<Vec<((f32, f32), usize)>>,
}

/// EnumHistogramStats are emtpy.
#[derive(Debug, PartialEq)]
pub struct EnumHistogramStats {}

/// Compute stats using the `dataset_stats` which contain histograms of the original data.
pub fn compute_histogram_stats(
	dataset_stats: &DatasetStats,
	progress: impl Fn(),
) -> HistogramStats {
	match dataset_stats {
		DatasetStats::Unknown(_) => HistogramStats::Unknown(UnknownHistogramStats {}),
		DatasetStats::Number(dataset_stats) => {
			HistogramStats::Number(compute_number_histogram_stats(
				&dataset_stats.histogram,
				dataset_stats.valid_count,
				progress,
			))
		}
		DatasetStats::Enum(_) => HistogramStats::Enum(EnumHistogramStats {}),
		DatasetStats::Text(_) => HistogramStats::Text(TextHistogramStats {}),
	}
}

/// Compute stats for number columns using the `dataset_stats` which contain histograms of the original data.
fn compute_number_histogram_stats(
	histogram: &BTreeMap<Finite<f32>, usize>,
	total_values_count: usize,
	progress: impl Fn(),
) -> NumberHistogramStats {
	let min = histogram.iter().next().unwrap().0.get();
	let max = histogram.iter().next_back().unwrap().0.get();
	let total_values_count = total_values_count.to_f32().unwrap();
	let quantiles: Vec<f32> = vec![0.25, 0.50, 0.75];
	// find the index of each quantile given the total number of values in the dataset
	let quantile_indexes: Vec<usize> = quantiles
		.iter()
		.map(|q| ((total_values_count - 1.0) * q).trunc().to_usize().unwrap())
		.collect();
	// the fractiononal part of the index
	// used to interpolate values if the index is not an integer value
	let quantile_fracts: Vec<f32> = quantiles
		.iter()
		.map(|q| ((total_values_count - 1.0) * q).fract())
		.collect();
	let mut quantiles: Vec<Option<f32>> = vec![None; quantiles.len()];
	let mut current_count: usize = 0;
	let mut mean = 0.0;
	let mut m2 = 0.0;
	let mut iter = histogram.iter().peekable();
	while let Some((value, count)) = iter.next() {
		let value = value.get();
		let (new_mean, new_m2) = metrics::merge_mean_m2(
			current_count.to_u64().unwrap(),
			mean,
			m2,
			count.to_u64().unwrap(),
			value.to_f64().unwrap(),
			0.0,
		);
		mean = new_mean;
		m2 = new_m2;
		current_count += count;
		let quantiles_iter = quantiles
			.iter_mut()
			.zip(quantile_indexes.iter().zip(quantile_fracts.iter()))
			.filter(|(q, (_, _))| q.is_none());
		for (quantile, (index, fract)) in quantiles_iter {
			match (current_count - 1).cmp(index) {
				Ordering::Equal => {
					if *fract > 0.0 {
						// interpolate between two values
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
		progress();
	}
	let quantiles: Vec<f32> = quantiles.into_iter().map(|q| q.unwrap()).collect();
	NumberHistogramStats {
		p25: quantiles[0],
		p50: quantiles[1],
		p75: quantiles[2],
		min,
		max,
		binned_histogram: None,
		mean: mean.to_f32().unwrap(),
		variance: metrics::m2_to_variance(
			m2,
			NonZeroU64::new(current_count.to_u64().unwrap()).unwrap(),
		),
	}
}

#[test]
fn test_compute_number_histogram_stats_one() {
	let mut histogram = BTreeMap::new();
	histogram.insert(Finite::new(1.0).unwrap(), 1);
	let left = compute_number_histogram_stats(&histogram, 1, || {});
	let right = NumberHistogramStats {
		min: 1.0,
		max: 1.0,
		p25: 1.0,
		p50: 1.0,
		p75: 1.0,
		mean: 1.0,
		variance: 0.0,
		binned_histogram: None,
	};
	assert_eq!(left, right);
}

#[test]
fn test_compute_number_histogram_stats_two() {
	let mut histogram = BTreeMap::new();
	histogram.insert(Finite::new(1.0).unwrap(), 1);
	histogram.insert(Finite::new(2.0).unwrap(), 1);
	let left = compute_number_histogram_stats(&histogram, 2, || {});
	let right = NumberHistogramStats {
		min: 1.0,
		max: 2.0,
		p25: 1.25,
		p50: 1.5,
		p75: 1.75,
		mean: 1.5,
		variance: 0.25,
		binned_histogram: None,
	};
	assert_eq!(left, right);
}

#[test]
fn test_compute_number_histogram_stats_multiple() {
	let mut histogram = BTreeMap::new();
	histogram.insert(Finite::new(1.0).unwrap(), 3);
	histogram.insert(Finite::new(2.0).unwrap(), 1);
	let left = compute_number_histogram_stats(&histogram, 4, || {});
	let right = NumberHistogramStats {
		min: 1.0,
		max: 2.0,
		p25: 1.0,
		p50: 1.0,
		p75: 1.25,
		mean: 1.25,
		variance: 0.1875,
		binned_histogram: None,
	};
	assert_eq!(left, right);
}
