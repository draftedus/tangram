/*!
This crate provides a basic implementation of dataframes, which are two dimensional arrays of data where each column can have a different data type, like a spreadsheet. This crate is similar to Python's Pandas library, but at present it is very limited, because it only implements the features needed to support Tangram.
*/

use fnv::FnvHashMap;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use std::num::NonZeroUsize;
use tangram_util::zip;

mod load;

pub use self::load::*;

pub mod prelude {
	pub use super::{
		DataFrame, DataFrameColumn, DataFrameColumnType, DataFrameColumnView, DataFrameValue,
		DataFrameView, DataFrameViewMut, EnumDataFrameColumn, EnumDataFrameColumnView,
		NumberDataFrameColumn, NumberDataFrameColumnView, TextDataFrameColumn,
		TextDataFrameColumnView, TextDataFrameColumnViewMut, UnknownDataFrameColumn,
		UnknownDataFrameColumnView,
	};
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataFrame {
	columns: Vec<DataFrameColumn>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataFrameView<'a> {
	columns: Vec<DataFrameColumnView<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct DataFrameViewMut<'a> {
	columns: Vec<DataFrameColumnViewMut<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataFrameColumn {
	Unknown(UnknownDataFrameColumn),
	Number(NumberDataFrameColumn),
	Enum(EnumDataFrameColumn),
	Text(TextDataFrameColumn),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnknownDataFrameColumn {
	name: Option<String>,
	len: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberDataFrameColumn {
	name: Option<String>,
	data: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDataFrameColumn {
	name: Option<String>,
	options: Vec<String>,
	data: Vec<Option<NonZeroUsize>>,
	options_map: FnvHashMap<String, NonZeroUsize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextDataFrameColumn {
	name: Option<String>,
	data: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataFrameColumnView<'a> {
	Unknown(UnknownDataFrameColumnView<'a>),
	Number(NumberDataFrameColumnView<'a>),
	Enum(EnumDataFrameColumnView<'a>),
	Text(TextDataFrameColumnView<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnknownDataFrameColumnView<'a> {
	name: Option<&'a str>,
	len: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberDataFrameColumnView<'a> {
	name: Option<&'a str>,
	data: &'a [f32],
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDataFrameColumnView<'a> {
	name: Option<&'a str>,
	options: &'a [String],
	data: &'a [Option<NonZeroUsize>],
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextDataFrameColumnView<'a> {
	name: Option<&'a str>,
	data: &'a [String],
}

#[derive(Debug, PartialEq)]
pub enum DataFrameColumnViewMut<'a> {
	Number(NumberDataFrameColumnViewMut<'a>),
	Enum(EnumDataFrameColumnViewMut<'a>),
	Text(TextDataFrameColumnViewMut<'a>),
}

#[derive(Debug, PartialEq)]
pub struct NumberDataFrameColumnViewMut<'a> {
	name: Option<&'a mut str>,
	data: &'a mut [f32],
}

#[derive(Debug, PartialEq)]
pub struct EnumDataFrameColumnViewMut<'a> {
	name: Option<&'a mut str>,
	options: &'a mut [String],
	data: &'a mut [usize],
}

#[derive(Debug, PartialEq)]
pub struct TextDataFrameColumnViewMut<'a> {
	name: Option<&'a mut str>,
	data: &'a mut [String],
}

#[derive(Debug, Clone)]
pub enum DataFrameColumnType {
	Unknown,
	Number,
	Enum { options: Vec<String> },
	Text,
}

#[derive(Debug, Clone)]
pub enum DataFrameColumnTypeView<'a> {
	Unknown,
	Number,
	Enum { options: &'a [String] },
	Text,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataFrameValue<'a> {
	Unknown,
	Number(f32),
	Enum(Option<NonZeroUsize>),
	Text(&'a str),
}

impl DataFrame {
	pub fn new(
		column_names: Vec<Option<String>>,
		column_types: Vec<DataFrameColumnType>,
	) -> DataFrame {
		let columns = zip!(column_names, column_types)
			.map(|(column_name, column_type)| match column_type {
				DataFrameColumnType::Unknown => {
					DataFrameColumn::Unknown(UnknownDataFrameColumn::new(column_name))
				}
				DataFrameColumnType::Number => {
					DataFrameColumn::Number(NumberDataFrameColumn::new(column_name, Vec::new()))
				}
				DataFrameColumnType::Enum { options } => DataFrameColumn::Enum(
					EnumDataFrameColumn::new(column_name, options, Vec::new()),
				),
				DataFrameColumnType::Text => {
					DataFrameColumn::Text(TextDataFrameColumn::new(column_name, Vec::new()))
				}
			})
			.collect();
		DataFrame { columns }
	}

	pub fn columns(&self) -> &Vec<DataFrameColumn> {
		&self.columns
	}

	pub fn columns_mut(&mut self) -> &mut Vec<DataFrameColumn> {
		&mut self.columns
	}

	pub fn ncols(&self) -> usize {
		self.columns.len()
	}

	pub fn nrows(&self) -> usize {
		self.columns.first().map(|column| column.len()).unwrap_or(0)
	}

	pub fn view(&self) -> DataFrameView {
		let columns = self.columns.iter().map(|column| column.view()).collect();
		DataFrameView { columns }
	}

	pub fn shuffle(&mut self, seed: u64) {
		for column in self.columns_mut().iter_mut() {
			let mut rng = Xoshiro256Plus::seed_from_u64(seed);
			match column {
				DataFrameColumn::Unknown(_) => {}
				DataFrameColumn::Number(column) => column.data_mut().shuffle(&mut rng),
				DataFrameColumn::Enum(column) => column.data_mut().shuffle(&mut rng),
				DataFrameColumn::Text(column) => column.data_mut().shuffle(&mut rng),
			}
		}
	}

	pub fn to_rows_f32(&self) -> Option<Array2<f32>> {
		let mut features_train = Array::zeros((self.nrows(), self.ncols()));
		for (mut ndarray_column, dataframe_column) in
			zip!(features_train.axis_iter_mut(Axis(1)), self.columns.iter())
		{
			match dataframe_column {
				DataFrameColumn::Number(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = *b;
					}
				}
				DataFrameColumn::Enum(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = b.map(|b| b.get().to_f32().unwrap()).unwrap_or(0.0);
					}
				}
				_ => return None,
			}
		}
		Some(features_train)
	}

	pub fn to_rows(&self) -> Array2<DataFrameValue> {
		let mut rows = Array::from_elem((self.nrows(), self.ncols()), DataFrameValue::Unknown);
		for (mut ndarray_column, dataframe_column) in
			zip!(rows.axis_iter_mut(Axis(1)), self.columns.iter())
		{
			match dataframe_column {
				DataFrameColumn::Unknown(_) => ndarray_column.fill(DataFrameValue::Unknown),
				DataFrameColumn::Number(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = DataFrameValue::Number(*b);
					}
				}
				DataFrameColumn::Enum(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = DataFrameValue::Enum(*b);
					}
				}
				DataFrameColumn::Text(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = DataFrameValue::Text(b);
					}
				}
			}
		}
		rows
	}
}

impl DataFrameColumn {
	pub fn len(&self) -> usize {
		match self {
			DataFrameColumn::Unknown(s) => s.len(),
			DataFrameColumn::Number(s) => s.len(),
			DataFrameColumn::Enum(s) => s.len(),
			DataFrameColumn::Text(s) => s.len(),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			DataFrameColumn::Unknown(s) => s.len == 0,
			DataFrameColumn::Number(s) => s.data.is_empty(),
			DataFrameColumn::Enum(s) => s.data.is_empty(),
			DataFrameColumn::Text(s) => s.data.is_empty(),
		}
	}

	pub fn name(&self) -> Option<&str> {
		match self {
			DataFrameColumn::Unknown(s) => s.name.as_deref(),
			DataFrameColumn::Number(s) => s.name.as_deref(),
			DataFrameColumn::Enum(s) => s.name.as_deref(),
			DataFrameColumn::Text(s) => s.name.as_deref(),
		}
	}

	pub fn as_number(&self) -> Option<&NumberDataFrameColumn> {
		match self {
			DataFrameColumn::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<&EnumDataFrameColumn> {
		match self {
			DataFrameColumn::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<&TextDataFrameColumn> {
		match self {
			DataFrameColumn::Text(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_number_mut(&mut self) -> Option<&mut NumberDataFrameColumn> {
		match self {
			DataFrameColumn::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum_mut(&mut self) -> Option<&mut EnumDataFrameColumn> {
		match self {
			DataFrameColumn::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text_mut(&mut self) -> Option<&mut TextDataFrameColumn> {
		match self {
			DataFrameColumn::Text(s) => Some(s),
			_ => None,
		}
	}

	pub fn view(&self) -> DataFrameColumnView {
		match self {
			DataFrameColumn::Unknown(column) => DataFrameColumnView::Unknown(column.view()),
			DataFrameColumn::Number(column) => DataFrameColumnView::Number(column.view()),
			DataFrameColumn::Enum(column) => DataFrameColumnView::Enum(column.view()),
			DataFrameColumn::Text(column) => DataFrameColumnView::Text(column.view()),
		}
	}
}

impl UnknownDataFrameColumn {
	pub fn new(name: Option<String>) -> UnknownDataFrameColumn {
		UnknownDataFrameColumn { name, len: 0 }
	}

	pub fn name(&self) -> &Option<String> {
		&self.name
	}

	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn len_mut(&mut self) -> &mut usize {
		&mut self.len
	}

	pub fn view(&self) -> UnknownDataFrameColumnView {
		UnknownDataFrameColumnView {
			name: self.name.as_deref(),
			len: self.len,
		}
	}
}

impl NumberDataFrameColumn {
	pub fn new(name: Option<String>, data: Vec<f32>) -> NumberDataFrameColumn {
		NumberDataFrameColumn { name, data }
	}

	pub fn name(&self) -> &Option<String> {
		&self.name
	}

	pub fn is_empty(&self) -> bool {
		self.data.len() == 0
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn iter(&self) -> impl Iterator<Item = &f32> {
		self.data.iter()
	}

	pub fn data_mut(&mut self) -> &mut Vec<f32> {
		&mut self.data
	}

	pub fn view(&self) -> NumberDataFrameColumnView {
		NumberDataFrameColumnView {
			name: self.name.as_deref(),
			data: &self.data,
		}
	}
}

impl EnumDataFrameColumn {
	pub fn new(
		name: Option<String>,
		options: Vec<String>,
		data: Vec<Option<NonZeroUsize>>,
	) -> EnumDataFrameColumn {
		let options_map = options
			.iter()
			.cloned()
			.enumerate()
			.map(|(i, option)| (option, NonZeroUsize::new(i + 1).unwrap()))
			.collect();
		EnumDataFrameColumn {
			name,
			options,
			data,
			options_map,
		}
	}

	pub fn name(&self) -> &Option<String> {
		&self.name
	}

	pub fn options(&self) -> &[String] {
		&self.options
	}

	pub fn is_empty(&self) -> bool {
		self.data.len() == 0
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn iter(&self) -> impl Iterator<Item = &Option<NonZeroUsize>> {
		self.data.iter()
	}

	pub fn data_mut(&mut self) -> &mut Vec<Option<NonZeroUsize>> {
		&mut self.data
	}

	pub fn view(&self) -> EnumDataFrameColumnView {
		EnumDataFrameColumnView {
			name: self.name.as_deref(),
			data: &self.data,
			options: &self.options,
		}
	}

	pub fn value_for_option(&self, option: &str) -> Option<NonZeroUsize> {
		self.options_map.get(option).cloned()
	}
}

impl TextDataFrameColumn {
	pub fn new(name: Option<String>, data: Vec<String>) -> TextDataFrameColumn {
		TextDataFrameColumn { name, data }
	}

	pub fn name(&self) -> &Option<String> {
		&self.name
	}

	pub fn is_empty(&self) -> bool {
		self.data.len() == 0
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn iter(&self) -> impl Iterator<Item = &String> {
		self.data.iter()
	}

	pub fn data_mut(&mut self) -> &mut Vec<String> {
		&mut self.data
	}

	pub fn view(&self) -> TextDataFrameColumnView {
		TextDataFrameColumnView {
			name: self.name.as_deref(),
			data: &self.data,
		}
	}
}

impl<'a> DataFrameView<'a> {
	pub fn columns(&self) -> &Vec<DataFrameColumnView<'a>> {
		&self.columns
	}

	pub fn subview(&self, indexes: &[usize]) -> DataFrameView {
		let mut columns = Vec::with_capacity(indexes.len());
		for index in indexes {
			columns.push(self.columns[*index].clone())
		}
		Self { columns }
	}

	pub fn ncols(&self) -> usize {
		self.columns.len()
	}

	pub fn nrows(&self) -> usize {
		self.columns.first().map(|column| column.len()).unwrap_or(0)
	}

	pub fn view(&self) -> DataFrameView {
		self.clone()
	}

	pub fn read_row(&self, index: usize, row: &mut [DataFrameValue<'a>]) {
		for (value, column) in zip!(row.iter_mut(), self.columns.iter()) {
			*value = match column {
				DataFrameColumnView::Unknown(_) => DataFrameValue::Unknown,
				DataFrameColumnView::Number(column) => DataFrameValue::Number(column.data[index]),
				DataFrameColumnView::Enum(column) => DataFrameValue::Enum(column.data[index]),
				DataFrameColumnView::Text(column) => DataFrameValue::Text(&column.data[index]),
			}
		}
	}

	pub fn split_at_row(&self, index: usize) -> (DataFrameView<'a>, DataFrameView<'a>) {
		let iter = self.columns.iter().map(|column| column.split_at_row(index));
		let mut columns_a = Vec::with_capacity(self.columns.len());
		let mut columns_b = Vec::with_capacity(self.columns.len());
		for (column_a, column_b) in iter {
			columns_a.push(column_a);
			columns_b.push(column_b);
		}
		(
			DataFrameView { columns: columns_a },
			DataFrameView { columns: columns_b },
		)
	}

	pub fn to_rows_f32(&self) -> Option<Array2<f32>> {
		let mut features_train = Array::zeros((self.nrows(), self.ncols()));
		for (mut ndarray_column, dataframe_column) in
			zip!(features_train.axis_iter_mut(Axis(1)), self.columns.iter())
		{
			match dataframe_column {
				DataFrameColumnView::Number(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data) {
						*a = *b;
					}
				}
				DataFrameColumnView::Enum(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data) {
						*a = b.unwrap().get().to_f32().unwrap();
					}
				}
				_ => return None,
			}
		}
		Some(features_train)
	}

	pub fn to_rows(&self) -> Array2<DataFrameValue<'a>> {
		let mut rows = Array::from_elem((self.nrows(), self.ncols()), DataFrameValue::Unknown);
		for (mut ndarray_column, dataframe_column) in
			zip!(rows.axis_iter_mut(Axis(1)), self.columns.iter())
		{
			match dataframe_column {
				DataFrameColumnView::Unknown(_) => ndarray_column.fill(DataFrameValue::Unknown),
				DataFrameColumnView::Number(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data) {
						*a = DataFrameValue::Number(*b);
					}
				}
				DataFrameColumnView::Enum(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data) {
						*a = DataFrameValue::Enum(*b);
					}
				}
				DataFrameColumnView::Text(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data) {
						*a = DataFrameValue::Text(b);
					}
				}
			}
		}
		rows
	}
}

impl<'a> DataFrameColumnView<'a> {
	pub fn len(&self) -> usize {
		match self {
			DataFrameColumnView::Unknown(s) => s.len,
			DataFrameColumnView::Number(s) => s.data.len(),
			DataFrameColumnView::Enum(s) => s.data.len(),
			DataFrameColumnView::Text(s) => s.data.len(),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			DataFrameColumnView::Unknown(s) => s.len == 0,
			DataFrameColumnView::Number(s) => s.data.is_empty(),
			DataFrameColumnView::Enum(s) => s.data.is_empty(),
			DataFrameColumnView::Text(s) => s.data.is_empty(),
		}
	}

	pub fn name(&self) -> Option<&str> {
		match self {
			DataFrameColumnView::Unknown(s) => s.name,
			DataFrameColumnView::Number(s) => s.name,
			DataFrameColumnView::Enum(s) => s.name,
			DataFrameColumnView::Text(s) => s.name,
		}
	}

	pub fn column_type(&self) -> DataFrameColumnTypeView {
		match self {
			DataFrameColumnView::Unknown(_) => DataFrameColumnTypeView::Unknown,
			DataFrameColumnView::Number(_) => DataFrameColumnTypeView::Number,
			DataFrameColumnView::Enum(column) => DataFrameColumnTypeView::Enum {
				options: column.options,
			},
			DataFrameColumnView::Text(_) => DataFrameColumnTypeView::Text,
		}
	}

	pub fn as_number(&self) -> Option<NumberDataFrameColumnView> {
		match self {
			DataFrameColumnView::Number(s) => Some(s.clone()),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<EnumDataFrameColumnView> {
		match self {
			DataFrameColumnView::Enum(s) => Some(s.clone()),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<TextDataFrameColumnView> {
		match self {
			DataFrameColumnView::Text(s) => Some(s.clone()),
			_ => None,
		}
	}

	pub fn split_at_row(&self, index: usize) -> (DataFrameColumnView<'a>, DataFrameColumnView<'a>) {
		match self {
			DataFrameColumnView::Unknown(column) => (
				DataFrameColumnView::Unknown(UnknownDataFrameColumnView {
					name: column.name,
					len: index,
				}),
				DataFrameColumnView::Unknown(UnknownDataFrameColumnView {
					name: column.name,
					len: column.len - index,
				}),
			),
			DataFrameColumnView::Number(column) => {
				let (data_a, data_b) = column.data.split_at(index);
				(
					DataFrameColumnView::Number(NumberDataFrameColumnView {
						name: column.name,
						data: data_a,
					}),
					DataFrameColumnView::Number(NumberDataFrameColumnView {
						name: column.name,
						data: data_b,
					}),
				)
			}
			DataFrameColumnView::Enum(column) => {
				let (data_a, data_b) = column.data.split_at(index);
				(
					DataFrameColumnView::Enum(EnumDataFrameColumnView {
						name: column.name,
						options: column.options,
						data: data_a,
					}),
					DataFrameColumnView::Enum(EnumDataFrameColumnView {
						name: column.name,
						options: column.options,
						data: data_b,
					}),
				)
			}
			DataFrameColumnView::Text(column) => {
				let (data_a, data_b) = column.data.split_at(index);
				(
					DataFrameColumnView::Text(TextDataFrameColumnView {
						name: column.name,
						data: data_a,
					}),
					DataFrameColumnView::Text(TextDataFrameColumnView {
						name: column.name,
						data: data_b,
					}),
				)
			}
		}
	}

	pub fn view(&self) -> DataFrameColumnView {
		match self {
			DataFrameColumnView::Unknown(s) => DataFrameColumnView::Unknown(s.view()),
			DataFrameColumnView::Number(s) => DataFrameColumnView::Number(s.view()),
			DataFrameColumnView::Enum(s) => DataFrameColumnView::Enum(s.view()),
			DataFrameColumnView::Text(s) => DataFrameColumnView::Text(s.view()),
		}
	}
}

impl<'a> UnknownDataFrameColumnView<'a> {
	pub fn name(&self) -> Option<&str> {
		self.name
	}

	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn view(&self) -> UnknownDataFrameColumnView {
		self.clone()
	}
}

impl<'a> NumberDataFrameColumnView<'a> {
	pub fn name(&self) -> Option<&str> {
		self.name
	}

	pub fn data(&self) -> &[f32] {
		self.data
	}

	pub fn is_empty(&self) -> bool {
		self.data.len() == 0
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn iter(&self) -> impl Iterator<Item = &f32> {
		self.data.iter()
	}

	pub fn as_slice(&self) -> &[f32] {
		self.data
	}

	pub fn view(&self) -> NumberDataFrameColumnView {
		self.clone()
	}
}

impl<'a> EnumDataFrameColumnView<'a> {
	pub fn name(&self) -> Option<&str> {
		self.name
	}

	pub fn data(&self) -> &[Option<NonZeroUsize>] {
		self.data
	}

	pub fn options(&self) -> &[String] {
		&self.options
	}

	pub fn is_empty(&self) -> bool {
		self.data.len() == 0
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn iter(&self) -> impl Iterator<Item = &Option<NonZeroUsize>> {
		self.data.iter()
	}

	pub fn as_slice(&self) -> &[Option<NonZeroUsize>] {
		self.data
	}

	pub fn view(&self) -> EnumDataFrameColumnView {
		self.clone()
	}
}

impl<'a> TextDataFrameColumnView<'a> {
	pub fn name(&self) -> Option<&str> {
		self.name
	}

	pub fn data(&self) -> &'a [String] {
		self.data
	}

	pub fn is_empty(&self) -> bool {
		self.data.len() == 0
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn iter(&self) -> impl Iterator<Item = &String> {
		self.data.iter()
	}

	pub fn as_slice(&self) -> &[String] {
		self.data
	}

	pub fn view(&self) -> TextDataFrameColumnView {
		self.clone()
	}
}

impl<'a> DataFrameValue<'a> {
	pub fn as_number(&self) -> Option<&f32> {
		match self {
			DataFrameValue::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_number_mut(&mut self) -> Option<&mut f32> {
		match self {
			DataFrameValue::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<&Option<NonZeroUsize>> {
		match self {
			DataFrameValue::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum_mut(&mut self) -> Option<&mut Option<NonZeroUsize>> {
		match self {
			DataFrameValue::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<&str> {
		match self {
			DataFrameValue::Text(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text_mut(&mut self) -> Option<&mut &'a str> {
		match self {
			DataFrameValue::Text(s) => Some(s),
			_ => None,
		}
	}
}
