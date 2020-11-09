use ndarray::prelude::*;
use num_traits::ToPrimitive;
use tangram_dataframe::{DataFrameColumnView, NumberDataFrameColumn};
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
			DataFrameColumnView::Enum(column) => {
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
			DataFrameColumnView::Number(column) => {
				let mean_variance =
					tangram_metrics::MeanVariance::compute(column.view().as_slice());
				Self {
					source_column_name: column.name().unwrap().to_owned(),
					mean: mean_variance.mean,
					variance: mean_variance.variance,
				}
			}
			_ => unimplemented!(),
		}
	}

	pub fn compute_array_f32(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: DataFrameColumnView,
		progress: &impl Fn(),
	) {
		// Set the feature values to the normalized source column values.
		match column {
			DataFrameColumnView::Unknown(_) => todo!(),
			DataFrameColumnView::Number(column) => {
				for (feature, value) in zip!(features.iter_mut(), column.iter()) {
					*feature = if value.is_nan() || self.variance == 0.0 {
						0.0
					} else {
						(*value - self.mean) / f32::sqrt(self.variance)
					};
					progress()
				}
			}
			DataFrameColumnView::Enum(column) => {
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
			DataFrameColumnView::Text(_) => todo!(),
		}
	}

	pub fn compute_dataframe(
		&self,
		mut feature: NumberDataFrameColumn,
		column: DataFrameColumnView,
		progress: &impl Fn(),
	) {
		// Set the feature values to the normalized source column values.
		match column {
			DataFrameColumnView::Unknown(_) => unimplemented!(),
			DataFrameColumnView::Number(column) => {
				for (feature, value) in zip!(feature.iter_mut(), column.iter()) {
					*feature = if value.is_nan() || self.variance == 0.0 {
						0.0
					} else {
						(*value - self.mean) / f32::sqrt(self.variance)
					};
					progress()
				}
			}
			DataFrameColumnView::Enum(column) => {
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
			DataFrameColumnView::Text(_) => todo!(),
		}
	}

	pub fn compute_array_value() {
		todo!()
	}
}
