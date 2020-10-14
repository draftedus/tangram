use super::*;
use anyhow::Result;
use std::{
	collections::{BTreeMap, BTreeSet},
	path::Path,
};

#[derive(Clone)]
pub struct FromCsvOptions<'a> {
	pub column_types: Option<BTreeMap<String, DataFrameColumnType>>,
	pub infer_options: InferOptions,
	pub invalid_values: &'a [&'a str],
}

impl<'a> Default for FromCsvOptions<'a> {
	fn default() -> Self {
		Self {
			column_types: None,
			infer_options: InferOptions::default(),
			invalid_values: DEFAULT_INVALID_VALUES,
		}
	}
}

#[derive(Clone, Debug)]
pub struct InferOptions {
	pub enum_max_unique_values: usize,
}

impl Default for InferOptions {
	fn default() -> Self {
		Self {
			enum_max_unique_values: 100,
		}
	}
}

/// These values are the default values that are considered invalid.
const DEFAULT_INVALID_VALUES: &[&str] = &[
	"", "null", "NULL", "n/a", "N/A", "nan", "-nan", "NaN", "-NaN", "?",
];

impl DataFrame {
	pub fn from_path(path: &Path, options: FromCsvOptions, progress: impl Fn(u64)) -> Result<Self> {
		Self::from_csv(&mut csv::Reader::from_path(path)?, options, progress)
	}

	pub fn from_csv<R>(
		reader: &mut csv::Reader<R>,
		options: FromCsvOptions,
		progress: impl Fn(u64),
	) -> Result<Self>
	where
		R: std::io::Read + std::io::Seek,
	{
		let column_names: Vec<String> = reader
			.headers()?
			.into_iter()
			.map(|column_name| column_name.to_owned())
			.collect();
		let n_columns = column_names.len();
		let start_position = reader.position().clone();
		let infer_options = &options.infer_options;
		let mut n_rows = None;

		#[derive(Clone, Debug)]
		enum ColumnTypeOrInferStats<'a> {
			ColumnType(DataFrameColumnType),
			InferStats(InferStats<'a>),
		}

		// Retrieve any column types present in the options.
		let mut column_types: Vec<ColumnTypeOrInferStats> = if let Some(column_types) =
			options.column_types
		{
			column_names
				.iter()
				.map(|column_name| {
					column_types
						.get(column_name)
						.map(|column_type| ColumnTypeOrInferStats::ColumnType(column_type.clone()))
						.unwrap_or_else(|| {
							ColumnTypeOrInferStats::InferStats(InferStats::new(infer_options))
						})
				})
				.collect()
		} else {
			vec![
				ColumnTypeOrInferStats::InferStats(InferStats::new(&options.infer_options));
				n_columns
			]
		};

		// Passing over the csv to infer column types is only necessary if one or more columns did not have its type specified.
		let needs_infer =
			column_types.iter().any(
				|column_type_or_infer_stats| match column_type_or_infer_stats {
					ColumnTypeOrInferStats::ColumnType(_) => false,
					ColumnTypeOrInferStats::InferStats(_) => true,
				},
			);

		// If the infer pass is necessary, pass over the dataset and infer the types for those columns whose types were not specified.
		let column_types: Vec<DataFrameColumnType> = if needs_infer {
			let mut infer_stats: Vec<(usize, &mut InferStats)> = column_types
				.iter_mut()
				.enumerate()
				.filter_map(
					|(index, column_type_or_infer_stats)| match column_type_or_infer_stats {
						ColumnTypeOrInferStats::ColumnType(_) => None,
						ColumnTypeOrInferStats::InferStats(infer_stats) => {
							Some((index, infer_stats))
						}
					},
				)
				.collect();
			// Iterate over each record in the csv file and update the infer stats for the columns that need to be inferred.
			let mut record = csv::StringRecord::new();
			let mut n_rows_computed = 0;
			while reader.read_record(&mut record)? {
				n_rows_computed += 1;
				for (index, infer_stats) in infer_stats.iter_mut() {
					let value = record.get(*index).unwrap();
					infer_stats.update(value);
				}
			}
			n_rows = Some(n_rows_computed);
			let column_types = column_types
				.into_iter()
				.map(
					|column_type_or_infer_stats| match column_type_or_infer_stats {
						ColumnTypeOrInferStats::ColumnType(column_type) => column_type,
						ColumnTypeOrInferStats::InferStats(infer_stats) => infer_stats.finalize(),
					},
				)
				.collect();
			// After inference, return back to the beginning of the csv to load the values.
			reader.seek(start_position)?;
			column_types
		} else {
			column_types
				.into_iter()
				.map(
					|column_type_or_infer_stats| match column_type_or_infer_stats {
						ColumnTypeOrInferStats::ColumnType(column_type) => column_type,
						_ => unreachable!(),
					},
				)
				.collect()
		};

		// Create the dataframe.
		let mut dataframe = Self::new(column_names, column_types);
		// If an inference pass was done, reserve storage for the values because we know how many rows are in the csv.
		if let Some(n_rows) = n_rows {
			for column in dataframe.columns.iter_mut() {
				match column {
					DataFrameColumn::Unknown(_) => {}
					DataFrameColumn::Number(column) => column.data.reserve_exact(n_rows),
					DataFrameColumn::Enum(column) => column.data.reserve_exact(n_rows),
					DataFrameColumn::Text(column) => column.data.reserve_exact(n_rows),
				}
			}
		}
		// Read each csv record and insert the values into the columns of the dataframe.
		let mut record = csv::ByteRecord::new();
		while reader.read_byte_record(&mut record)? {
			progress(record.position().unwrap().byte());
			for (column, value) in dataframe.columns.iter_mut().zip(record.iter()) {
				match column {
					DataFrameColumn::Unknown(column) => {
						column.len += 1;
					}
					DataFrameColumn::Number(column) => {
						let value = match lexical::parse::<f32, &[u8]>(value) {
							Ok(value) if value.is_finite() => value,
							_ => std::f32::NAN,
						};
						column.data.push(value);
					}
					DataFrameColumn::Enum(column) => {
						let value = if let Ok(value) = std::str::from_utf8(value) {
							column
								.options
								.get(value)
								.map(|position| Some(NonZeroUsize::new(*position + 1).unwrap()))
								.unwrap_or(None)
						} else {
							None
						};
						column.data.push(value);
					}
					DataFrameColumn::Text(column) => {
						column.data.push(std::str::from_utf8(value)?.to_owned())
					}
				}
			}
		}
		Ok(dataframe)
	}
}

#[derive(Clone, Debug)]
pub struct InferStats<'a> {
	infer_options: &'a InferOptions,
	column_type: InferColumnType,
	unique_values: Option<BTreeSet<String>>,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum InferColumnType {
	Unknown,
	Number,
	Enum,
	Text,
}

impl<'a> InferStats<'a> {
	pub fn new(infer_options: &'a InferOptions) -> Self {
		Self {
			infer_options,
			column_type: InferColumnType::Unknown,
			unique_values: Some(BTreeSet::new()),
		}
	}

	pub fn update(&mut self, value: &str) {
		if DEFAULT_INVALID_VALUES.contains(&value) {
			return;
		}
		if let Some(unique_values) = self.unique_values.as_mut() {
			if !unique_values.contains(value) {
				unique_values.insert(value.to_owned());
			}
			if unique_values.len() > self.infer_options.enum_max_unique_values {
				self.unique_values = None;
			}
		}
		match self.column_type {
			InferColumnType::Unknown | InferColumnType::Number => {
				if lexical::parse::<f32, &str>(value)
					.map(|v| v.is_finite())
					.unwrap_or(false)
				{
					self.column_type = InferColumnType::Number;
				} else if self.unique_values.is_some() {
					self.column_type = InferColumnType::Enum;
				} else {
					self.column_type = InferColumnType::Text;
				}
			}
			InferColumnType::Enum => {
				if self.unique_values.is_none() {
					self.column_type = InferColumnType::Text;
				}
			}
			_ => {}
		}
	}

	pub fn finalize(self) -> DataFrameColumnType {
		match self.column_type {
			InferColumnType::Unknown => DataFrameColumnType::Unknown,
			InferColumnType::Number => {
				// If all the values in a number column are zero or one then make this an enum column instead.
				if let Some(unique_values) = self.unique_values {
					let mut values = unique_values.iter();
					if values.next().map(|s| s.as_str()) == Some("0")
						&& values.next().map(|s| s.as_str()) == Some("1")
					{
						return DataFrameColumnType::Enum {
							options: unique_values.into_iter().collect(),
						};
					}
				}
				DataFrameColumnType::Number
			}
			InferColumnType::Enum => DataFrameColumnType::Enum {
				options: self.unique_values.unwrap().into_iter().collect(),
			},
			InferColumnType::Text => DataFrameColumnType::Text,
		}
	}
}

#[test]
fn test_infer() {
	let csv = r#"number,enum,text
1,test,hello
2,test,world
"#;
	let df = DataFrame::from_csv(
		&mut csv::Reader::from_reader(std::io::Cursor::new(csv)),
		FromCsvOptions {
			column_types: None,
			infer_options: InferOptions {
				enum_max_unique_values: 1,
			},
			..Default::default()
		},
		Box::new(|_| {}),
	)
	.unwrap();
	insta::assert_debug_snapshot!(df, @r###"
 DataFrame {
     columns: [
         Number(
             NumberColumn {
                 name: "number",
                 data: [
                     1.0,
                     2.0,
                 ],
             },
         ),
         Enum(
             EnumColumn {
                 name: "enum",
                 options: [
                     "test",
                 ],
                 data: [
                     Some(
                         1,
                     ),
                     Some(
                         1,
                     ),
                 ],
             },
         ),
         Text(
             TextColumn {
                 name: "text",
                 data: [
                     "hello",
                     "world",
                 ],
             },
         ),
     ],
 }
 "###);
}

#[test]
fn test_column_types() {
	let csv = r#"number,text,enum
1,test,hello
2,test,world
"#;
	let mut column_types = BTreeMap::new();
	column_types.insert("text".to_owned(), DataFrameColumnType::Text);
	column_types.insert(
		"enum".to_owned(),
		DataFrameColumnType::Enum {
			options: vec!["hello".to_owned(), "world".to_owned()],
		},
	);
	let df = DataFrame::from_csv(
		&mut csv::Reader::from_reader(std::io::Cursor::new(csv)),
		FromCsvOptions {
			column_types: Some(column_types),
			infer_options: InferOptions {
				enum_max_unique_values: 2,
			},
			..Default::default()
		},
		Box::new(|_| {}),
	)
	.unwrap();
	insta::assert_debug_snapshot!(df, @r###"
 DataFrame {
     columns: [
         Number(
             NumberColumn {
                 name: "number",
                 data: [
                     1.0,
                     2.0,
                 ],
             },
         ),
         Text(
             TextColumn {
                 name: "text",
                 data: [
                     "test",
                     "test",
                 ],
             },
         ),
         Enum(
             EnumColumn {
                 name: "enum",
                 options: [
                     "hello",
                     "world",
                 ],
                 data: [
                     Some(
                         1,
                     ),
                     Some(
                         2,
                     ),
                 ],
             },
         ),
     ],
 }
 "###);
}
