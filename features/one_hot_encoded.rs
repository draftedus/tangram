use ndarray::prelude::*;
use tangram_dataframe::{DataFrameColumnView, DataFrameValue, NumberDataFrameColumn};
use tangram_util::zip;

/**
A `OneHotEncodedFeatureGroup` creates one number feature for each option in an enum column, plus one number feature for invalid values. For each example, all of the features will have the value 0.0, except the feature corresponding to the column's value, which will have the value 1.0.

# Example

```
use std::num::NonZeroUsize;
use tangram_dataframe::prelude::*;

EnumDataFrameColumn::new(
  Some("color".to_string()),
  vec!["red".to_string(), "green".to_string(), "blue".to_string()],
  vec![None, Some(NonZeroUsize::new(1).unwrap()), Some(NonZeroUsize::new(2).unwrap()), Some(NonZeroUsize::new(3).unwrap())],
);
```

| dataframe value | feature values |
|-----------------|----------------|
| "INVALID!"      | [0, 0, 0]      |
| "red"           | [1, 0, 0]      |
| "green"         | [0, 1, 0]      |
| "blue"          | [0, 0, 1]      |
*/
#[derive(Debug)]
pub struct OneHotEncodedFeatureGroup {
	pub source_column_name: String,
	pub options: Vec<String>,
}

impl OneHotEncodedFeatureGroup {
	pub fn fit(column: DataFrameColumnView) -> OneHotEncodedFeatureGroup {
		match column {
			DataFrameColumnView::Enum(column) => Self {
				source_column_name: column.name().unwrap().to_owned(),
				options: column.options().to_owned(),
			},
			_ => unimplemented!(),
		}
	}

	pub fn compute_array_f32(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: DataFrameColumnView,
		progress: &impl Fn(),
	) {
		match column {
			DataFrameColumnView::Enum(column) => {
				// Fill the features with zeros.
				features.fill(0.0);
				// For each example, set the features corresponding to the enum value to one.
				for (mut features, value) in
					zip!(features.axis_iter_mut(Axis(0)), column.as_slice().iter())
				{
					let feature_index = value.map(|v| v.get()).unwrap_or(0);
					features[feature_index] = 1.0;
					progress();
				}
			}
			DataFrameColumnView::Unknown(_) => unimplemented!(),
			DataFrameColumnView::Number(_) => unimplemented!(),
			DataFrameColumnView::Text(_) => unimplemented!(),
		}
	}

	pub fn compute_dataframe(
		&self,
		_features: &mut [NumberDataFrameColumn],
		_column: DataFrameColumnView,
		_progress: &impl Fn(),
	) {
		todo!()
	}

	pub fn compute_array_value(
		&self,
		mut _features: ArrayViewMut2<DataFrameValue>,
		_column: DataFrameColumnView,
		_progress: &impl Fn(),
	) {
		todo!()
	}
}
