/*!
This crate provides a basic implementation of dataframes, which are two dimensional arrays of data where each column can have a different data type, like a spreadsheet. This crate is similar to Python's Pandas library, but at present is incredibly limited, because it only implements the features needed to support Tangram.
*/

use fnv::FnvBuildHasher;
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::{collections::HashMap, num::NonZeroUsize};

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
	pub columns: Vec<DataFrameColumn>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataFrameView<'a> {
	pub columns: Vec<DataFrameColumnView<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct DataFrameViewMut<'a> {
	pub columns: Vec<DataFrameColumnViewMut<'a>>,
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
	pub name: Option<String>,
	pub len: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberDataFrameColumn {
	pub name: Option<String>,
	pub data: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDataFrameColumn {
	pub name: Option<String>,
	pub options: Vec<String>,
	pub data: Vec<Option<NonZeroUsize>>,
	options_map: HashMap<String, NonZeroUsize, FnvBuildHasher>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextDataFrameColumn {
	pub name: Option<String>,
	pub data: Vec<String>,
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
	pub name: Option<&'a str>,
	pub len: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberDataFrameColumnView<'a> {
	pub name: Option<&'a str>,
	pub data: &'a [f32],
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDataFrameColumnView<'a> {
	pub name: Option<&'a str>,
	pub options: &'a [String],
	pub data: &'a [Option<NonZeroUsize>],
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextDataFrameColumnView<'a> {
	pub name: Option<&'a str>,
	pub data: &'a [String],
}

#[derive(Debug, PartialEq)]
pub enum DataFrameColumnViewMut<'a> {
	Number(NumberDataFrameColumnViewMut<'a>),
	Enum(EnumDataFrameColumnViewMut<'a>),
	Text(TextDataFrameColumnViewMut<'a>),
}

#[derive(Debug, PartialEq)]
pub struct NumberDataFrameColumnViewMut<'a> {
	pub name: Option<&'a mut str>,
	pub data: &'a mut [f32],
}

#[derive(Debug, PartialEq)]
pub struct EnumDataFrameColumnViewMut<'a> {
	pub name: Option<&'a mut str>,
	pub options: &'a mut [String],
	pub data: &'a mut [usize],
}

#[derive(Debug, PartialEq)]
pub struct TextDataFrameColumnViewMut<'a> {
	pub name: Option<&'a mut str>,
	pub data: &'a mut [String],
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
	pub fn new(column_names: Vec<Option<String>>, column_types: Vec<DataFrameColumnType>) -> Self {
		let columns = column_names
			.into_iter()
			.zip(column_types.into_iter())
			.map(|(column_name, column_type)| match column_type {
				DataFrameColumnType::Unknown => {
					DataFrameColumn::Unknown(UnknownDataFrameColumn::new(column_name))
				}
				DataFrameColumnType::Number => {
					DataFrameColumn::Number(NumberDataFrameColumn::new(column_name))
				}
				DataFrameColumnType::Enum { options } => {
					DataFrameColumn::Enum(EnumDataFrameColumn::new(column_name, options))
				}
				DataFrameColumnType::Text => {
					DataFrameColumn::Text(TextDataFrameColumn::new(column_name))
				}
			})
			.collect();
		Self { columns }
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

	pub fn to_rows_f32(&self) -> Option<Array2<f32>> {
		let mut features_train = unsafe { Array::uninitialized((self.nrows(), self.ncols())) };
		for (mut ndarray_column, dataframe_column) in
			izip!(features_train.axis_iter_mut(Axis(1)), self.columns.iter())
		{
			match dataframe_column {
				DataFrameColumn::Number(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = *b;
					}
				}
				DataFrameColumn::Enum(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = b.unwrap().get().to_f32().unwrap();
					}
				}
				_ => return None,
			}
		}
		Some(features_train)
	}

	pub fn to_rows(&self) -> Array2<DataFrameValue> {
		let mut rows = unsafe { Array2::uninitialized((self.nrows(), self.ncols())) };
		for (mut ndarray_column, dataframe_column) in
			izip!(rows.axis_iter_mut(Axis(1)), self.columns.iter())
		{
			match dataframe_column {
				DataFrameColumn::Unknown(_) => ndarray_column.fill(DataFrameValue::Unknown),
				DataFrameColumn::Number(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = DataFrameValue::Number(*b);
					}
				}
				DataFrameColumn::Enum(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = DataFrameValue::Enum(*b);
					}
				}
				DataFrameColumn::Text(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data.as_slice()) {
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
			Self::Unknown(s) => s.len,
			Self::Number(s) => s.data.len(),
			Self::Enum(s) => s.data.len(),
			Self::Text(s) => s.data.len(),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			Self::Unknown(s) => s.len == 0,
			Self::Number(s) => s.data.is_empty(),
			Self::Enum(s) => s.data.is_empty(),
			Self::Text(s) => s.data.is_empty(),
		}
	}

	pub fn name(&self) -> Option<&str> {
		match self {
			Self::Unknown(s) => s.name.as_deref(),
			Self::Number(s) => s.name.as_deref(),
			Self::Enum(s) => s.name.as_deref(),
			Self::Text(s) => s.name.as_deref(),
		}
	}

	pub fn as_number(&self) -> Option<&NumberDataFrameColumn> {
		match self {
			Self::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<&EnumDataFrameColumn> {
		match self {
			Self::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<&TextDataFrameColumn> {
		match self {
			Self::Text(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_number_mut(&mut self) -> Option<&mut NumberDataFrameColumn> {
		match self {
			Self::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum_mut(&mut self) -> Option<&mut EnumDataFrameColumn> {
		match self {
			Self::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text_mut(&mut self) -> Option<&mut TextDataFrameColumn> {
		match self {
			Self::Text(s) => Some(s),
			_ => None,
		}
	}

	pub fn view(&self) -> DataFrameColumnView {
		match self {
			Self::Unknown(column) => DataFrameColumnView::Unknown(column.view()),
			Self::Number(column) => DataFrameColumnView::Number(column.view()),
			Self::Enum(column) => DataFrameColumnView::Enum(column.view()),
			Self::Text(column) => DataFrameColumnView::Text(column.view()),
		}
	}
}

impl UnknownDataFrameColumn {
	pub fn new(name: Option<String>) -> Self {
		Self { name, len: 0 }
	}

	pub fn view(&self) -> UnknownDataFrameColumnView {
		UnknownDataFrameColumnView {
			name: self.name.as_deref(),
			len: self.len,
		}
	}
}

impl NumberDataFrameColumn {
	pub fn new(name: Option<String>) -> Self {
		Self {
			name,
			data: Vec::new(),
		}
	}

	pub fn view(&self) -> NumberDataFrameColumnView {
		NumberDataFrameColumnView {
			name: self.name.as_deref(),
			data: &self.data,
		}
	}
}

impl EnumDataFrameColumn {
	pub fn new(name: Option<String>, options: Vec<String>) -> Self {
		let options_map = options
			.iter()
			.cloned()
			.enumerate()
			.map(|(i, option)| (option, NonZeroUsize::new(i + 1).unwrap()))
			.collect();
		Self {
			name,
			options,
			data: Vec::new(),
			options_map,
		}
	}

	pub fn options(&self) -> &[String] {
		&self.options
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
	pub fn new(name: Option<String>) -> Self {
		Self {
			name,
			data: Vec::new(),
		}
	}

	pub fn view(&self) -> TextDataFrameColumnView {
		TextDataFrameColumnView {
			name: self.name.as_deref(),
			data: &self.data,
		}
	}
}

impl<'a> DataFrameView<'a> {
	pub fn ncols(&self) -> usize {
		self.columns.len()
	}

	pub fn nrows(&self) -> usize {
		self.columns.first().map(|column| column.len()).unwrap_or(0)
	}

	pub fn view(&self) -> Self {
		self.clone()
	}

	pub fn read_row(&self, index: usize, row: &mut [DataFrameValue<'a>]) {
		for (value, column) in row.iter_mut().zip(self.columns.iter()) {
			*value = match column {
				DataFrameColumnView::Unknown(_) => DataFrameValue::Unknown,
				DataFrameColumnView::Number(column) => DataFrameValue::Number(column.data[index]),
				DataFrameColumnView::Enum(column) => DataFrameValue::Enum(column.data[index]),
				DataFrameColumnView::Text(column) => DataFrameValue::Text(&column.data[index]),
			}
		}
	}

	pub fn split_at_row(&self, index: usize) -> (Self, Self) {
		let iter = self.columns.iter().map(|column| column.split_at_row(index));
		let mut columns_a = Vec::with_capacity(self.columns.len());
		let mut columns_b = Vec::with_capacity(self.columns.len());
		for (column_a, column_b) in iter {
			columns_a.push(column_a);
			columns_b.push(column_b);
		}
		(Self { columns: columns_a }, Self { columns: columns_b })
	}

	pub fn to_rows_f32(&self) -> Option<Array2<f32>> {
		let mut features_train = unsafe { Array::uninitialized((self.nrows(), self.ncols())) };
		for (mut ndarray_column, dataframe_column) in
			izip!(features_train.axis_iter_mut(Axis(1)), self.columns.iter())
		{
			match dataframe_column {
				DataFrameColumnView::Number(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data) {
						*a = *b;
					}
				}
				DataFrameColumnView::Enum(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data) {
						*a = b.unwrap().get().to_f32().unwrap();
					}
				}
				_ => return None,
			}
		}
		Some(features_train)
	}

	pub fn to_rows(&self) -> Array2<DataFrameValue<'a>> {
		let mut rows = unsafe { Array2::uninitialized((self.nrows(), self.ncols())) };
		for (mut ndarray_column, dataframe_column) in
			izip!(rows.axis_iter_mut(Axis(1)), self.columns.iter())
		{
			match dataframe_column {
				DataFrameColumnView::Unknown(_) => ndarray_column.fill(DataFrameValue::Unknown),
				DataFrameColumnView::Number(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data) {
						*a = DataFrameValue::Number(*b);
					}
				}
				DataFrameColumnView::Enum(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data) {
						*a = DataFrameValue::Enum(*b);
					}
				}
				DataFrameColumnView::Text(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data) {
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
			Self::Unknown(s) => s.len,
			Self::Number(s) => s.data.len(),
			Self::Enum(s) => s.data.len(),
			Self::Text(s) => s.data.len(),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			Self::Unknown(s) => s.len == 0,
			Self::Number(s) => s.data.is_empty(),
			Self::Enum(s) => s.data.is_empty(),
			Self::Text(s) => s.data.is_empty(),
		}
	}

	pub fn name(&self) -> Option<&str> {
		match self {
			Self::Unknown(s) => s.name,
			Self::Number(s) => s.name,
			Self::Enum(s) => s.name,
			Self::Text(s) => s.name,
		}
	}

	pub fn column_type(&self) -> DataFrameColumnTypeView {
		match self {
			Self::Unknown(_) => DataFrameColumnTypeView::Unknown,
			Self::Number(_) => DataFrameColumnTypeView::Number,
			Self::Enum(column) => DataFrameColumnTypeView::Enum {
				options: column.options,
			},
			Self::Text(_) => DataFrameColumnTypeView::Text,
		}
	}

	pub fn as_number(&self) -> Option<NumberDataFrameColumnView> {
		match self {
			Self::Number(s) => Some(s.clone()),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<EnumDataFrameColumnView> {
		match self {
			Self::Enum(s) => Some(s.clone()),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<TextDataFrameColumnView> {
		match self {
			Self::Text(s) => Some(s.clone()),
			_ => None,
		}
	}

	pub fn split_at_row(&self, index: usize) -> (Self, Self) {
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

	pub fn view(&self) -> Self {
		match self {
			DataFrameColumnView::Unknown(s) => DataFrameColumnView::Unknown(s.view()),
			DataFrameColumnView::Number(s) => DataFrameColumnView::Number(s.view()),
			DataFrameColumnView::Enum(s) => DataFrameColumnView::Enum(s.view()),
			DataFrameColumnView::Text(s) => DataFrameColumnView::Text(s.view()),
		}
	}
}

impl<'a> UnknownDataFrameColumnView<'a> {
	pub fn view(&self) -> Self {
		self.clone()
	}
}

impl<'a> NumberDataFrameColumnView<'a> {
	pub fn view(&self) -> Self {
		self.clone()
	}
}

impl<'a> EnumDataFrameColumnView<'a> {
	pub fn options(&self) -> &[String] {
		&self.options
	}

	pub fn view(&self) -> Self {
		self.clone()
	}
}

impl<'a> TextDataFrameColumnView<'a> {
	pub fn view(&self) -> Self {
		self.clone()
	}
}

impl<'a> DataFrameValue<'a> {
	pub fn as_number(&self) -> Option<&f32> {
		match self {
			Self::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_number_mut(&mut self) -> Option<&mut f32> {
		match self {
			Self::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<&Option<NonZeroUsize>> {
		match self {
			Self::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum_mut(&mut self) -> Option<&mut Option<NonZeroUsize>> {
		match self {
			Self::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<&str> {
		match self {
			Self::Text(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text_mut(&mut self) -> Option<&mut &'a str> {
		match self {
			Self::Text(s) => Some(s),
			_ => None,
		}
	}
}
