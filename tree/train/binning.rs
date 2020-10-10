use crate::TrainOptions;
use itertools::izip;
use itertools::Itertools;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use rayon::prelude::*;
use std::{cmp::Ordering, collections::BTreeMap};
use tangram_dataframe::{ColumnView, DataFrameView, NumberColumnView};
use tangram_finite::Finite;
use tangram_thread_pool::pzip;

/*
This struct specifies how to bin a feature.

## Number
Number features have the first bin reserved for invalid values, and after that feature values are binned by comparing them with a set of thresholds. For example, given the thresholds `[0.5, 1.5, 2]`, the bins will be:
0. invalid values
1. (-infinity, 0.5]
2. (0.5, 1.5]
3. (1.5, 2]
4. (2, infinity)

## Enum
Enum features have one bin for each enum option. For example, gives the options `["A", "B", "C"]`, the bins will be:
0. invalid values
1. "A"
2. "B"
3. "C"
*/
#[derive(Clone, Debug)]
pub enum BinningInstructions {
	Number { thresholds: Vec<f32> },
	Enum { n_options: usize },
}

impl BinningInstructions {
	pub fn n_bins(&self) -> usize {
		1 + self.n_valid_bins()
	}
	pub fn n_valid_bins(&self) -> usize {
		match self {
			Self::Number { thresholds } => thresholds.len() + 1,
			Self::Enum { n_options } => *n_options,
		}
	}
}

/// Compute the binning instructions for each column in `features`.
pub fn compute_binning_instructions(
	features: &DataFrameView,
	train_options: &TrainOptions,
) -> Vec<BinningInstructions> {
	features
		.columns
		.par_iter()
		.map(|column| match column.view() {
			ColumnView::Number(column) => {
				compute_binning_instructions_for_number_feature(column, &train_options)
			}
			ColumnView::Enum(column) => BinningInstructions::Enum {
				n_options: column.options.len(),
			},
			_ => unreachable!(),
		})
		.collect()
}

/// Compute the binning instructions for a number feature.
fn compute_binning_instructions_for_number_feature(
	column: NumberColumnView,
	train_options: &TrainOptions,
) -> BinningInstructions {
	// Create a histogram of values in the number feature.
	let mut histogram: BTreeMap<Finite<f32>, usize> = BTreeMap::new();
	let mut histogram_values_count = 0;
	for value in &column.data[0..column
		.data
		.len()
		.min(train_options.max_examples_for_computing_bin_thresholds)]
	{
		if let Ok(value) = Finite::new(*value) {
			*histogram.entry(value).or_insert(0) += 1;
			histogram_values_count += 1;
		}
	}
	// If the number of unique values is less than `max_valid_bins_for_number_features`, then create one bin per unique value. Otherwise, create bins at quantiles.
	let thresholds = if histogram.len()
		< train_options
			.max_valid_bins_for_number_features
			.to_usize()
			.unwrap()
	{
		histogram
			.keys()
			.tuple_windows()
			.map(|(a, b)| (a.get() + b.get()) / 2.0)
			.collect()
	} else {
		compute_binning_instruction_thresholds_for_number_feature_as_quantiles_from_histogram(
			histogram,
			histogram_values_count,
			train_options,
		)
	};
	BinningInstructions::Number { thresholds }
}

/// Compute the binning instruction thresholds for a number feature as quantiles from the histogram of its values.
fn compute_binning_instruction_thresholds_for_number_feature_as_quantiles_from_histogram(
	histogram: BTreeMap<Finite<f32>, usize>,
	histogram_values_count: usize,
	train_options: &TrainOptions,
) -> Vec<f32> {
	let total_values_count = histogram_values_count.to_f32().unwrap();
	let quantiles: Vec<f32> = (1..train_options
		.max_valid_bins_for_number_features
		.to_usize()
		.unwrap())
		.map(|i| {
			i.to_f32().unwrap()
				/ train_options
					.max_valid_bins_for_number_features
					.to_f32()
					.unwrap()
		})
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

/// A dataframe of features is binned into BinnedFeatures
#[derive(Debug)]
pub enum BinnedFeatures {
	RowWise(RowWiseBinnedFeatures),
	ColWise(ColWiseBinnedFeatures),
}

#[derive(Debug)]
pub struct RowWiseBinnedFeatures {
	pub values_with_offsets: Array2<u16>,
	pub offsets: Vec<u16>,
}

#[derive(Debug)]
pub struct ColWiseBinnedFeatures {
	pub columns: Vec<ColWiseBinnedFeaturesColumn>,
}

#[derive(Debug)]
pub enum ColWiseBinnedFeaturesColumn {
	U8(Vec<u8>),
	U16(Vec<u16>),
}

impl ColWiseBinnedFeaturesColumn {
	pub fn len(&self) -> usize {
		match self {
			ColWiseBinnedFeaturesColumn::U8(values) => values.len(),
			ColWiseBinnedFeaturesColumn::U16(values) => values.len(),
		}
	}
}

/// Compute the binned features based on the binning instructions.
pub fn compute_binned_features(
	features: &DataFrameView,
	binning_instructions: &[BinningInstructions],
	progress: &(impl Fn() + Sync),
) -> BinnedFeatures {
	let columns = pzip!(&features.columns, binning_instructions)
		.map(
			|(feature, binning_instructions)| match binning_instructions {
				BinningInstructions::Number { thresholds } => {
					compute_binned_features_for_number_feature(feature, thresholds, progress)
				}
				BinningInstructions::Enum { n_options } => {
					if *n_options <= 255 {
						compute_binned_features_for_enum_feature_u8(feature, progress)
					} else {
						compute_binned_features_for_enum_feature_u16(feature, progress)
					}
				}
			},
		)
		.collect();
	BinnedFeatures::ColWise(ColWiseBinnedFeatures { columns })
}

pub fn compute_binned_features_row_wise(
	features: &DataFrameView,
	binning_instructions: &[BinningInstructions],
	_progress: &(impl Fn() + Sync),
) -> BinnedFeatures {
	let total_bins = binning_instructions
		.iter()
		.map(|binning_instructions| binning_instructions.n_bins())
		.sum::<usize>();
	if total_bins < 15 {
		todo!();
	} else if total_bins < 255 {
		todo!()
	} else if total_bins < 65536 {
		// bin type is u16
		let n_features = features.ncols();
		let n_examples = features.nrows();
		let mut values_with_offsets: Array2<u16> =
			unsafe { Array2::uninitialized((n_examples, n_features)) };
		let mut current_offset = 0;
		let offsets: Vec<u16> = binning_instructions
			.iter()
			.map(|binning_instructions| {
				let this_offset = current_offset.to_u16().unwrap();
				current_offset += binning_instructions.n_bins();
				this_offset
			})
			.collect();
		pzip!(
			values_with_offsets.axis_iter_mut(Axis(1)),
			&features.columns,
			binning_instructions,
			offsets.as_slice()
		)
		.for_each(
			|(mut binned_features_column, feature, binning_instructions, offset)| {
				match binning_instructions {
					BinningInstructions::Number { thresholds } => {
						izip!(
							binned_features_column.iter_mut(),
							feature.as_number().unwrap().data
						)
						.for_each(|(binned_feature_value, feature_value)| {
							// Invalid values go to the first bin.
							if !feature_value.is_finite() {
								*binned_feature_value = *offset;
							}
							// Use binary search on the thresholds to find the bin for the feature value.
							*binned_feature_value = offset
								+ thresholds
									.binary_search_by(|threshold| {
										threshold.partial_cmp(feature_value).unwrap()
									})
									.unwrap_or_else(|bin| bin)
									.to_u16()
									.unwrap() + 1;
						});
					}
					BinningInstructions::Enum { .. } => {
						izip!(
							binned_features_column.iter_mut(),
							feature.as_enum().unwrap().data
						)
						.for_each(|(binned_feature_value, feature_value)| {
							*binned_feature_value = offset
								+ feature_value
									.map(|v| v.get())
									.unwrap_or(0)
									.to_u16()
									.unwrap();
						});
					}
				}
			},
		);
		BinnedFeatures::RowWise(RowWiseBinnedFeatures {
			values_with_offsets,
			offsets,
		})
	} else {
		todo!()
	}
}

fn compute_binned_features_for_number_feature(
	feature: &ColumnView,
	thresholds: &[f32],
	_progress: &(impl Fn() + Sync),
) -> ColWiseBinnedFeaturesColumn {
	let binned_feature = feature
		.as_number()
		.unwrap()
		.data
		.par_iter()
		.map(|feature_value| {
			// Invalid values go to the first bin.
			if !feature_value.is_finite() {
				return 0;
			}
			// Use binary search on the thresholds to find the bin for the feature value.
			thresholds
				.binary_search_by(|threshold| threshold.partial_cmp(feature_value).unwrap())
				.unwrap_or_else(|bin| bin)
				.to_u8()
				.unwrap() + 1
		})
		.collect::<Vec<u8>>();
	ColWiseBinnedFeaturesColumn::U8(binned_feature)
}

fn compute_binned_features_for_enum_feature_u8(
	feature: &ColumnView,
	_progress: &(impl Fn() + Sync),
) -> ColWiseBinnedFeaturesColumn {
	let binned_feature = feature
		.as_enum()
		.unwrap()
		.data
		.par_iter()
		.map(|feature_value| feature_value.map(|v| v.get()).unwrap_or(0).to_u8().unwrap())
		.collect::<Vec<u8>>();
	ColWiseBinnedFeaturesColumn::U8(binned_feature)
}

fn compute_binned_features_for_enum_feature_u16(
	feature: &ColumnView,
	_progress: &(impl Fn() + Sync),
) -> ColWiseBinnedFeaturesColumn {
	let binned_feature = feature
		.as_enum()
		.unwrap()
		.data
		.par_iter()
		.map(|feature_value| {
			feature_value
				.map(|v| v.get())
				.unwrap_or(0)
				.to_u16()
				.unwrap()
		})
		.collect::<Vec<u16>>();
	ColWiseBinnedFeaturesColumn::U16(binned_feature)
}
