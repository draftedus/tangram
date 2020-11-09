use ndarray::prelude::*;
use num_traits::ToPrimitive;
use tangram_dataframe::{
	DataFrameColumn, DataFrameColumnView, DataFrameValue, EnumDataFrameColumn,
	EnumDataFrameColumnView, NumberDataFrameColumn, NumberDataFrameColumnView,
};
use tangram_util::zip;

/**
An `IdentityFeatureGroup` describes the simplest possible feature engineering, which passes a single column from the input dataframe to the output features untouched.

# Example
For a number column:

| dataframe value | feature value |
|-----------------|---------------|
| 0.2             | 0.2           |
| 3.0             | 3.0           |
| 2.1             | 2.1           |

For an enum column:

```
use std::num::NonZeroUsize;
use tangram_dataframe::prelude::*;

EnumDataFrameColumn::new(
  Some("color".to_string()),
  vec!["red".to_string(), "green".to_string(), "blue".to_string()],
  vec![None, Some(NonZeroUsize::new(1).unwrap()), Some(NonZeroUsize::new(2).unwrap()), Some(NonZeroUsize::new(3).unwrap())],
);
```

| value       | encoding |
|-------------|----------|
| "INVALID!"  | None     |
| "red"       | Some(1)  |
| "green"     | Some(2)  |
| "blue"      | Some(3)  |

| dataframe value | feature value |
|-----------------|---------------|
| "INVALID!"      | None          |
| "red"           | Some(1)       |
| "green"         | Some(2)       |
| "blue"          | Some(3)       |
*/
#[derive(Debug)]
pub struct IdentityFeatureGroup {
	pub source_column_name: String,
}

impl IdentityFeatureGroup {
	pub fn compute_array_f32(
		&self,
		features: ArrayViewMut2<f32>,
		column: DataFrameColumnView,
		progress: &impl Fn(),
	) {
		// Set the feature values to the source column values.
		match column {
			DataFrameColumnView::Unknown(_) => todo!(),
			DataFrameColumnView::Number(column) => {
				self.compute_array_f32_for_number_column(features, column, progress)
			}
			DataFrameColumnView::Enum(column) => {
				self.compute_array_f32_for_enum_column(features, column, progress)
			}
			DataFrameColumnView::Text(_) => todo!(),
		}
	}

	fn compute_array_f32_for_number_column(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: NumberDataFrameColumnView,
		progress: &impl Fn(),
	) {
		for (feature, value) in zip!(features.iter_mut(), column.view().iter()) {
			*feature = *value;
			progress()
		}
	}

	fn compute_array_f32_for_enum_column(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: EnumDataFrameColumnView,
		progress: &impl Fn(),
	) {
		for (feature, value) in zip!(features.iter_mut(), column.view().iter()) {
			*feature = value.map(|v| v.get().to_f32().unwrap()).unwrap_or(0.0);
			progress()
		}
	}

	pub fn compute_dataframe(
		&self,
		column: DataFrameColumnView,
		progress: &impl Fn(u64),
	) -> DataFrameColumn {
		let column = match column {
			DataFrameColumnView::Unknown(_) => unimplemented!(),
			DataFrameColumnView::Number(column) => {
				DataFrameColumn::Number(self.compute_dataframe_for_number_column(column))
			}
			DataFrameColumnView::Enum(column) => {
				DataFrameColumn::Enum(self.compute_datafame_for_enum_column(column))
			}
			DataFrameColumnView::Text(_) => unimplemented!(),
		};
		progress(column.len().to_u64().unwrap());
		column
	}

	fn compute_dataframe_for_number_column(
		&self,
		column: NumberDataFrameColumnView,
	) -> NumberDataFrameColumn {
		NumberDataFrameColumn::new(
			column.name().map(|name| name.to_owned()),
			column.as_slice().to_owned(),
		)
	}

	fn compute_datafame_for_enum_column(
		&self,
		column: EnumDataFrameColumnView,
	) -> EnumDataFrameColumn {
		EnumDataFrameColumn::new(
			column.name().map(|name| name.to_owned()),
			column.options().to_owned(),
			column.as_slice().to_owned(),
		)
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
		for (feature_column, column_value) in zip!(features.column_mut(0), column.iter()) {
			*feature_column = DataFrameValue::Number(*column_value);
			progress()
		}
	}

	fn compute_array_value_for_enum_column(
		&self,
		mut features: ArrayViewMut2<DataFrameValue>,
		column: EnumDataFrameColumnView,
		progress: &impl Fn(),
	) {
		for (feature_column, column_value) in zip!(features.column_mut(0), column.iter()) {
			*feature_column = DataFrameValue::Enum(*column_value);
			progress()
		}
	}
}
