use ndarray::prelude::*;
use num_traits::ToPrimitive;
use tangram_dataframe::{
	DataFrameColumnView, DataFrameValue, EnumDataFrameColumnView, NumberDataFrameColumn,
	NumberDataFrameColumnView,
};
use tangram_metrics::Metric;
use tangram_util::zip;

/**
A `NormalizedFeatureGroup` transforms a number column to zero mean and unit variance. [Learn more](https://en.wikipedia.org/wiki/Feature_scaling#Standardization_(Z-score_Normalization).

# Example

```
use tangram_dataframe::prelude::*;

NumberDataFrameColumn::new(
  Some("values".to_string()),
  vec![0.0, 5.2, 1.3, 10.0],
);
```

Mean: 2.16667

Standard Deviation: 2.70617

`feature_value =  (value - mean) / std`

| dataframe value | feature value                         |
|-----------------|---------------------------------------|
| 0.0             | (0.0 - 2.16667) / 2.70617  = -0.80064 |
| 5.2             | (5.2 - 2.16667) / 2.70617  = 1.12089  |
| 1.3             | (1.3 - 2.16667) / 2.70617  = -0.32026 |
*/
#[derive(Debug)]
pub struct NormalizedFeatureGroup {
	pub source_column_name: String,
	pub mean: f32,
	pub variance: f32,
}

impl NormalizedFeatureGroup {
	pub fn fit(column: DataFrameColumnView) -> NormalizedFeatureGroup {
		match column {
			DataFrameColumnView::Number(column) => Self::fit_for_number_column(column),
			DataFrameColumnView::Enum(column) => Self::fit_for_enum_column(column),
			_ => unimplemented!(),
		}
	}

	fn fit_for_number_column(column: NumberDataFrameColumnView) -> Self {
		let mean_variance = tangram_metrics::MeanVariance::compute(column.view().as_slice());
		Self {
			source_column_name: column.name().unwrap().to_owned(),
			mean: mean_variance.mean,
			variance: mean_variance.variance,
		}
	}

	fn fit_for_enum_column(column: EnumDataFrameColumnView) -> Self {
		let values = column
			.as_slice()
			.iter()
			.map(|value| {
				value
					.map(|value| value.get().to_f32().unwrap())
					.unwrap_or(0.0)
			})
			.collect::<Vec<_>>();
		let mean_variance = tangram_metrics::MeanVariance::compute(values.as_slice());
		Self {
			source_column_name: column.name().unwrap().to_owned(),
			mean: mean_variance.mean,
			variance: mean_variance.variance,
		}
	}

	pub fn compute_array_f32(
		&self,
		features: ArrayViewMut2<f32>,
		column: DataFrameColumnView,
		progress: &impl Fn(),
	) {
		// Set the feature values to the normalized source column values.
		match column {
			DataFrameColumnView::Unknown(_) => unimplemented!(),
			DataFrameColumnView::Number(column) => {
				self.compute_array_f32_for_number_column(features, column, progress)
			}
			DataFrameColumnView::Enum(column) => {
				self.compute_array_f32_for_enum_column(features, column, progress)
			}
			DataFrameColumnView::Text(_) => unimplemented!(),
		}
	}

	fn compute_array_f32_for_number_column(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: NumberDataFrameColumnView,
		progress: &impl Fn(),
	) {
		for (feature, value) in zip!(features.iter_mut(), column.iter()) {
			*feature = if value.is_nan() || self.variance == 0.0 {
				0.0
			} else {
				(*value - self.mean) / f32::sqrt(self.variance)
			};
			progress()
		}
	}

	fn compute_array_f32_for_enum_column(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: EnumDataFrameColumnView,
		progress: &impl Fn(),
	) {
		for (feature, value) in zip!(features.iter_mut(), column.iter()) {
			let value = value
				.map(|value| value.get().to_f32().unwrap())
				.unwrap_or(0.0);
			*feature = if value.is_nan() || self.variance == 0.0 {
				0.0
			} else {
				(value - self.mean) / f32::sqrt(self.variance)
			};
			progress()
		}
	}

	pub fn compute_dataframe(
		&self,
		feature: NumberDataFrameColumn,
		column: DataFrameColumnView,
		progress: &impl Fn(),
	) {
		// Set the feature values to the normalized source column values.
		match column {
			DataFrameColumnView::Unknown(_) => unimplemented!(),
			DataFrameColumnView::Number(column) => {
				self.compute_dataframe_for_number_column(feature, column, progress)
			}
			DataFrameColumnView::Enum(column) => {
				self.compute_dataframe_for_enum_column(feature, column, progress)
			}
			DataFrameColumnView::Text(_) => todo!(),
		}
	}

	fn compute_dataframe_for_number_column(
		&self,
		mut feature: NumberDataFrameColumn,
		column: NumberDataFrameColumnView,
		progress: &impl Fn(),
	) {
		for (feature, value) in zip!(feature.iter_mut(), column.iter()) {
			*feature = if value.is_nan() || self.variance == 0.0 {
				0.0
			} else {
				(*value - self.mean) / f32::sqrt(self.variance)
			};
			progress()
		}
	}

	fn compute_dataframe_for_enum_column(
		&self,
		mut feature: NumberDataFrameColumn,
		column: EnumDataFrameColumnView,
		progress: &impl Fn(),
	) {
		for (feature, value) in zip!(feature.iter_mut(), column.iter()) {
			let value = value
				.map(|value| value.get().to_f32().unwrap())
				.unwrap_or(0.0);
			*feature = if value.is_nan() || self.variance == 0.0 {
				0.0
			} else {
				(value - self.mean) / f32::sqrt(self.variance)
			};
			progress()
		}
	}

	pub fn compute_array_value(
		&self,
		features: ArrayViewMut2<DataFrameValue>,
		column: DataFrameColumnView,
		progress: &impl Fn(),
	) {
		match column {
			DataFrameColumnView::Unknown(_) => unimplemented!(),
			DataFrameColumnView::Number(column) => {
				self.compute_array_value_for_number_column(features, column, progress)
			}
			DataFrameColumnView::Enum(column) => {
				self.compute_array_value_for_enum_column(features, column, progress)
			}
			DataFrameColumnView::Text(_) => unimplemented!(),
		}
	}

	fn compute_array_value_for_number_column(
		&self,
		mut features: ArrayViewMut2<DataFrameValue>,
		column: NumberDataFrameColumnView,
		progress: &impl Fn(),
	) {
		for (feature, value) in zip!(features.column_mut(0), column.iter()) {
			*feature = if value.is_nan() || self.variance == 0.0 {
				DataFrameValue::Number(0.0)
			} else {
				DataFrameValue::Number((value - self.mean) / f32::sqrt(self.variance))
			};
			progress()
		}
	}

	fn compute_array_value_for_enum_column(
		&self,
		mut features: ArrayViewMut2<DataFrameValue>,
		column: EnumDataFrameColumnView,
		progress: &impl Fn(),
	) {
		for (feature, value) in zip!(features.column_mut(0), column.iter()) {
			*feature = if value.is_none() || self.variance == 0.0 {
				DataFrameValue::Number(0.0)
			} else {
				DataFrameValue::Number(
					(value.unwrap().get().to_f32().unwrap() - self.mean) / f32::sqrt(self.variance),
				)
			};
			progress()
		}
	}
}
