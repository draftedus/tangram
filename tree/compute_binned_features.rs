use crate::compute_binning_instructions::BinningInstruction;
use ndarray::prelude::*;
use num_traits::{Num, NumCast, ToPrimitive};
use rayon::prelude::*;
use tangram_dataframe::{DataFrameColumnView, DataFrameView};
use tangram_util::pzip;

#[derive(Debug)]
pub enum BinnedFeaturesRowMajor {
	U16(BinnedFeaturesRowMajorInner<u16>),
	U32(BinnedFeaturesRowMajorInner<u32>),
}

#[derive(Debug)]
pub struct BinnedFeaturesRowMajorInner<T>
where
	T: NumCast,
{
	pub values_with_offsets: Array2<T>,
	pub offsets: Vec<T>,
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
		n_bins_across_all_features if n_bins_across_all_features <= 65536 => {
			BinnedFeaturesRowMajor::U16(compute_binned_features_row_major_inner(
				features,
				binning_instructions,
				progress,
			))
		}
		n_bins_across_all_features if n_bins_across_all_features <= 4294967296 => {
			BinnedFeaturesRowMajor::U32(compute_binned_features_row_major_inner(
				features,
				binning_instructions,
				progress,
			))
		}
		_ => unreachable!(),
	}
}

fn compute_binned_features_row_major_inner<T, P>(
	features: &DataFrameView,
	binning_instructions: &[BinningInstruction],
	_progress: &P,
) -> BinnedFeaturesRowMajorInner<T>
where
	T: Num + NumCast + Send + Sync + Copy + std::ops::Add + std::ops::AddAssign,
	P: Fn() + Sync,
{
	let n_features = features.ncols();
	let n_examples = features.nrows();
	let mut values_with_offsets: Array2<T> =
		unsafe { Array::uninitialized((n_examples, n_features)) };
	let mut offsets: Vec<T> = Vec::with_capacity(n_features);
	let mut current_offset: T = T::zero();
	for binning_instruction in binning_instructions.iter() {
		offsets.push(current_offset);
		current_offset += T::from(binning_instruction.n_bins()).unwrap();
	}
	pzip!(
		values_with_offsets.axis_iter_mut(Axis(1)),
		features.columns().as_slice(),
		binning_instructions,
		&offsets,
	)
	.for_each(
		|(mut binned_features_column, feature, binning_instruction, offset)| {
			match binning_instruction {
				BinningInstruction::Number { thresholds } => {
					pzip!(
						binned_features_column.axis_iter_mut(Axis(0)),
						feature.as_number().unwrap().as_slice(),
					)
					.for_each(|(binned_feature_value, feature_value)| {
						// Invalid values go to the first bin.
						let binned_feature_value = binned_feature_value.into_scalar();
						if !feature_value.is_finite() {
							*binned_feature_value = *offset;
						} else {
							let threshold = thresholds
								.binary_search_by(|threshold| {
									threshold.partial_cmp(feature_value).unwrap()
								})
								.unwrap_or_else(|bin| bin);
							let threshold = T::from(threshold).unwrap();
							// Use binary search on the thresholds to find the bin for the feature value.
							*binned_feature_value = *offset + threshold + T::one();
						}
					});
				}
				BinningInstruction::Enum { .. } => {
					pzip!(
						binned_features_column.axis_iter_mut(Axis(0)),
						feature.as_enum().unwrap().as_slice(),
					)
					.for_each(|(binned_feature_value, feature_value)| {
						let feature_value = feature_value.map(|v| v.get()).unwrap_or(0);
						let feature_value = T::from(feature_value).unwrap();
						*binned_feature_value.into_scalar() = *offset + feature_value;
					});
				}
			}
		},
	);
	BinnedFeaturesRowMajorInner {
		values_with_offsets,
		offsets,
	}
}

/// Compute the binned features based on the binning instructions.
pub fn compute_binned_features_column_major(
	features: &DataFrameView,
	binning_instructions: &[BinningInstruction],
	progress: &(impl Fn() + Sync),
) -> BinnedFeaturesColumnMajor {
	let columns = pzip!(features.columns().as_slice(), binning_instructions)
		.map(|(feature, binning_instruction)| match binning_instruction {
			BinningInstruction::Number { thresholds } => BinnedFeaturesColumnMajorColumn::U8(
				compute_binned_features_column_major_for_number_feature(
					feature, thresholds, progress,
				),
			),
			BinningInstruction::Enum { n_options } => {
				if *n_options <= 255 {
					BinnedFeaturesColumnMajorColumn::U8(
						compute_binned_features_column_major_for_enum_feature_inner(
							feature, progress,
						),
					)
				} else if *n_options <= 65535 {
					BinnedFeaturesColumnMajorColumn::U16(
						compute_binned_features_column_major_for_enum_feature_inner(
							feature, progress,
						),
					)
				} else {
					panic!("enum column has too many options")
				}
			}
		})
		.collect();
	BinnedFeaturesColumnMajor { columns }
}

fn compute_binned_features_column_major_for_number_feature(
	feature: &DataFrameColumnView,
	thresholds: &[f32],
	_progress: &(impl Fn() + Sync),
) -> Vec<u8> {
	let binned_feature_column = feature
		.as_number()
		.unwrap()
		.as_slice()
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
		.collect();
	binned_feature_column
}

fn compute_binned_features_column_major_for_enum_feature_inner<T, P>(
	feature: &DataFrameColumnView,
	_progress: &P,
) -> Vec<T>
where
	T: NumCast + Send + Sync,
	P: Fn() + Sync,
{
	feature
		.as_enum()
		.unwrap()
		.as_slice()
		.par_iter()
		.map(|feature_value| T::from(feature_value.map(|v| v.get()).unwrap_or(0)).unwrap())
		.collect()
}
