use crate::compute_binning_instructions::BinningInstruction;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use rayon::prelude::*;
use tangram_dataframe::{ColumnView, DataFrameView};
use tangram_thread_pool::pzip;

#[derive(Debug)]
pub struct BinnedFeaturesRowMajor {
	pub values_with_offsets: Array2<u16>,
	pub offsets: Vec<u16>,
}

#[derive(Debug)]
pub struct BinnedFeaturesColumnMajor {
	pub columns: Vec<BinnedFeaturesColumnMajorColumn>,
}

#[derive(Debug)]
pub enum BinnedFeaturesColumnMajorColumn {
	U8(Vec<u8>),
	U16(Vec<u16>),
}

impl BinnedFeaturesColumnMajorColumn {
	pub fn len(&self) -> usize {
		match self {
			BinnedFeaturesColumnMajorColumn::U8(values) => values.len(),
			BinnedFeaturesColumnMajorColumn::U16(values) => values.len(),
		}
	}
}

/// Compute the binned features based on the binning instructions.
pub fn compute_binned_features_column_major(
	features: &DataFrameView,
	binning_instructions: &[BinningInstruction],
	progress: &(impl Fn() + Sync),
) -> BinnedFeaturesColumnMajor {
	let columns = pzip!(&features.columns, binning_instructions)
		.map(|(feature, binning_instruction)| match binning_instruction {
			BinningInstruction::Number { thresholds } => {
				compute_binned_features_for_number_feature(feature, thresholds, progress)
			}
			BinningInstruction::Enum { n_options } => {
				if *n_options <= 255 {
					compute_binned_features_for_enum_feature_u8(feature, progress)
				} else {
					compute_binned_features_for_enum_feature_u16(feature, progress)
				}
			}
		})
		.collect();
	BinnedFeaturesColumnMajor { columns }
}

pub fn compute_binned_features_row_major(
	features: &DataFrameView,
	binning_instructions: &[BinningInstruction],
	progress: &(impl Fn() + Sync),
) -> BinnedFeaturesRowMajor {
	let n_bins_across_all_features = binning_instructions
		.iter()
		.map(|binning_instructions| binning_instructions.n_bins())
		.sum::<usize>();
	match n_bins_across_all_features {
		n_bins_across_all_features if n_bins_across_all_features < 1 << 16 => {
			compute_binned_features_row_major_u16(features, binning_instructions, progress)
		}
		_ => unreachable!(),
	}
}

fn compute_binned_features_row_major_u16(
	features: &DataFrameView,
	binning_instructions: &[BinningInstruction],
	_progress: &(impl Fn() + Sync),
) -> BinnedFeaturesRowMajor {
	let n_features = features.ncols();
	let n_examples = features.nrows();
	let mut values_with_offsets: Array2<u16> =
		unsafe { Array2::uninitialized((n_examples, n_features)) };
	let mut offsets: Vec<u16> = Vec::with_capacity(n_features);
	let mut current_offset: u16 = 0;
	for binning_instruction in binning_instructions.iter() {
		offsets.push(current_offset);
		current_offset += binning_instruction.n_bins().to_u16().unwrap();
	}
	pzip!(
		values_with_offsets.axis_iter_mut(Axis(1)),
		&features.columns,
		binning_instructions,
		&offsets,
	)
	.for_each(
		|(mut binned_features_column, feature, binning_instruction, offset)| {
			match binning_instruction {
				BinningInstruction::Number { thresholds } => {
					pzip!(
						binned_features_column.axis_iter_mut(Axis(0)),
						feature.as_number().unwrap().data
					)
					.for_each(|(binned_feature_value, feature_value)| {
						// Invalid values go to the first bin.
						let binned_feature_value = binned_feature_value.into_scalar();
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
				BinningInstruction::Enum { .. } => {
					pzip!(
						binned_features_column.axis_iter_mut(Axis(0)),
						feature.as_enum().unwrap().data
					)
					.for_each(|(binned_feature_value, feature_value)| {
						*binned_feature_value.into_scalar() = offset
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
	BinnedFeaturesRowMajor {
		values_with_offsets,
		offsets,
	}
}

fn compute_binned_features_for_number_feature(
	feature: &ColumnView,
	thresholds: &[f32],
	_progress: &(impl Fn() + Sync),
) -> BinnedFeaturesColumnMajorColumn {
	let binned_feature_column = feature
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
	BinnedFeaturesColumnMajorColumn::U8(binned_feature_column)
}

fn compute_binned_features_for_enum_feature_u8(
	feature: &ColumnView,
	_progress: &(impl Fn() + Sync),
) -> BinnedFeaturesColumnMajorColumn {
	let binned_feature_column = feature
		.as_enum()
		.unwrap()
		.data
		.par_iter()
		.map(|feature_value| feature_value.map(|v| v.get()).unwrap_or(0).to_u8().unwrap())
		.collect::<Vec<u8>>();
	BinnedFeaturesColumnMajorColumn::U8(binned_feature_column)
}

fn compute_binned_features_for_enum_feature_u16(
	feature: &ColumnView,
	_progress: &(impl Fn() + Sync),
) -> BinnedFeaturesColumnMajorColumn {
	let binned_feature_column = feature
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
	BinnedFeaturesColumnMajorColumn::U16(binned_feature_column)
}
