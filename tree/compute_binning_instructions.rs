use crate::TrainOptions;
use itertools::{izip, Itertools};
use num_traits::ToPrimitive;
use rayon::prelude::*;
use std::{cmp::Ordering, collections::BTreeMap};
use tangram_dataframe::{DataFrameColumnView, DataFrameView, NumberDataFrameColumnView};
use tangram_util::finite::Finite;

/*
This struct specifies how to produce a binned feature from a feature.

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
pub enum BinningInstruction {
	Number { thresholds: Vec<f32> },
	Enum { n_options: usize },
}

impl BinningInstruction {
	pub fn n_bins(&self) -> usize {
		1 + self.n_valid_bins()
	}
	pub fn n_valid_bins(&self) -> usize {
		match self {
			BinningInstruction::Number { thresholds } => thresholds.len() + 1,
			BinningInstruction::Enum { n_options } => *n_options,
		}
	}
}

/// Compute the binning instructions for each column in `features`.
pub fn compute_binning_instructions(
	features: &DataFrameView,
	train_options: &TrainOptions,
) -> Vec<BinningInstruction> {
	features
		.columns()
		.par_iter()
		.map(|column| match column.view() {
			DataFrameColumnView::Number(column) => {
				compute_binning_instructions_for_number_feature(column, &train_options)
			}
			DataFrameColumnView::Enum(column) => BinningInstruction::Enum {
				n_options: column.options().len(),
			},
			_ => unreachable!(),
		})
		.collect()
}

/// Compute the binning instructions for a number feature.
fn compute_binning_instructions_for_number_feature(
	column: NumberDataFrameColumnView,
	train_options: &TrainOptions,
) -> BinningInstruction {
	// Create a histogram of values in the number feature.
	let mut histogram: BTreeMap<Finite<f32>, usize> = BTreeMap::new();
	let mut histogram_values_count = 0;
	let max = usize::min(
		column.len(),
		train_options.max_examples_for_computing_bin_thresholds,
	);
	for value in column.iter().take(max) {
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
	BinningInstruction::Number { thresholds }
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
		let quantiles_iter = izip!(
			quantiles.iter_mut(),
			quantile_indexes.iter(),
			quantile_fracts.iter()
		)
		.filter(|(quantile, _, _)| quantile.is_none());
		for (quantile, index, fract) in quantiles_iter {
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
