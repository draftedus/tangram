/*!
This crate provides a basic implementation of dataframes, which are two dimensional arrays of data where each column can have a different data type, like a spreadsheet. This crate is similar to Python's Pandas library, but at present is incredibly limited, because it only implements the features needed to support Tangram.
*/

use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::num::NonZeroUsize;

pub mod load;

pub use self::load::*;

#[derive(Debug, Clone, PartialEq)]
pub struct DataFrame {
	pub columns: Vec<Column>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataFrameView<'a> {
	pub columns: Vec<ColumnView<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct DataFrameViewMut<'a> {
	pub columns: Vec<ColumnViewMut<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Column {
	Unknown(UnknownColumn),
	Number(NumberColumn),
	Enum(EnumColumn),
	Text(TextColumn),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnknownColumn {
	pub name: String,
	pub len: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberColumn {
	pub name: String,
	pub data: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumColumn {
	pub name: String,
	pub options: Vec<String>,
	pub data: Vec<Option<NonZeroUsize>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextColumn {
	pub name: String,
	pub data: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnView<'a> {
	Unknown(UnknownColumnView<'a>),
	Number(NumberColumnView<'a>),
	Enum(EnumColumnView<'a>),
	Text(TextColumnView<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnknownColumnView<'a> {
	pub name: &'a str,
	pub len: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberColumnView<'a> {
	pub name: &'a str,
	pub data: &'a [f32],
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumColumnView<'a> {
	pub name: &'a str,
	pub options: &'a [String],
	pub data: &'a [Option<NonZeroUsize>],
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextColumnView<'a> {
	pub name: &'a str,
	pub data: &'a [String],
}

#[derive(Debug, PartialEq)]
pub enum ColumnViewMut<'a> {
	Number(NumberColumnViewMut<'a>),
	Enum(EnumColumnViewMut<'a>),
	Text(TextColumnViewMut<'a>),
}

#[derive(Debug, PartialEq)]
pub struct NumberColumnViewMut<'a> {
	pub name: &'a mut str,
	pub data: &'a mut [f32],
}

#[derive(Debug, PartialEq)]
pub struct EnumColumnViewMut<'a> {
	pub name: &'a mut str,
	pub options: &'a mut [String],
	pub data: &'a mut [usize],
}

#[derive(Debug, PartialEq)]
pub struct TextColumnViewMut<'a> {
	pub name: &'a mut str,
	pub data: &'a mut [String],
}

#[derive(Debug, Clone)]
pub enum ColumnType {
	Unknown,
	Number,
	Enum { options: Vec<String> },
	Text,
}

#[derive(Debug, Clone)]
pub enum ColumnTypeView<'a> {
	Unknown,
	Number,
	Enum { options: &'a [String] },
	Text,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value<'a> {
	Unknown,
	Number(f32),
	Enum(Option<NonZeroUsize>),
	Text(&'a str),
}

impl DataFrame {
	pub fn new(column_names: Vec<String>, column_types: Vec<ColumnType>) -> Self {
		let columns = column_names
			.into_iter()
			.zip(column_types.into_iter())
			.map(|(column_name, column_type)| match column_type {
				ColumnType::Unknown => Column::Unknown(UnknownColumn::new(column_name)),
				ColumnType::Number => Column::Number(NumberColumn::new(column_name)),
				ColumnType::Enum { options } => Column::Enum(EnumColumn::new(column_name, options)),
				ColumnType::Text => Column::Text(TextColumn::new(column_name)),
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
			izip!(features_train.gencolumns_mut(), self.columns.iter())
		{
			match dataframe_column {
				Column::Number(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = *b;
					}
				}
				Column::Enum(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = b.unwrap().get().to_f32().unwrap();
					}
				}
				_ => return None,
			}
		}
		Some(features_train)
	}

	pub fn to_rows(&self) -> Array2<Value> {
		let mut rows = unsafe { Array2::uninitialized((self.nrows(), self.ncols())) };
		for (mut ndarray_column, dataframe_column) in
			izip!(rows.gencolumns_mut(), self.columns.iter())
		{
			match dataframe_column {
				Column::Unknown(_) => ndarray_column.fill(Value::Unknown),
				Column::Number(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = Value::Number(*b);
					}
				}
				Column::Enum(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = Value::Enum(*b);
					}
				}
				Column::Text(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = Value::Text(b);
					}
				}
			}
		}
		rows
	}
}

impl Column {
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

	pub fn name(&self) -> &str {
		match self {
			Self::Unknown(s) => s.name.as_str(),
			Self::Number(s) => s.name.as_str(),
			Self::Enum(s) => s.name.as_str(),
			Self::Text(s) => s.name.as_str(),
		}
	}

	pub fn as_number(&self) -> Option<&NumberColumn> {
		match self {
			Self::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<&EnumColumn> {
		match self {
			Self::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<&TextColumn> {
		match self {
			Self::Text(s) => Some(s),
			_ => None,
		}
	}

	pub fn view(&self) -> ColumnView {
		match self {
			Self::Unknown(column) => ColumnView::Unknown(column.view()),
			Self::Number(column) => ColumnView::Number(column.view()),
			Self::Enum(column) => ColumnView::Enum(column.view()),
			Self::Text(column) => ColumnView::Text(column.view()),
		}
	}
}

impl UnknownColumn {
	pub fn new(name: String) -> Self {
		Self { name, len: 0 }
	}

	pub fn view(&self) -> UnknownColumnView {
		UnknownColumnView {
			name: &self.name,
			len: self.len,
		}
	}
}

impl NumberColumn {
	pub fn new(name: String) -> Self {
		Self {
			name,
			data: Vec::new(),
		}
	}

	pub fn view(&self) -> NumberColumnView {
		NumberColumnView {
			name: &self.name,
			data: &self.data,
		}
	}
}

impl EnumColumn {
	pub fn new(name: String, options: Vec<String>) -> Self {
		Self {
			name,
			options,
			data: Vec::new(),
		}
	}

	pub fn view(&self) -> EnumColumnView {
		EnumColumnView {
			name: &self.name,
			data: &self.data,
			options: &self.options,
		}
	}
}

impl TextColumn {
	pub fn new(name: String) -> Self {
		Self {
			name,
			data: Vec::new(),
		}
	}

	pub fn view(&self) -> TextColumnView {
		TextColumnView {
			name: &self.name,
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

	pub fn read_row(&self, index: usize, row: &mut [Value<'a>]) {
		for (value, column) in row.iter_mut().zip(self.columns.iter()) {
			*value = match column {
				ColumnView::Unknown(_) => Value::Unknown,
				ColumnView::Number(column) => Value::Number(column.data[index]),
				ColumnView::Enum(column) => Value::Enum(column.data[index]),
				ColumnView::Text(column) => Value::Text(&column.data[index]),
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
			izip!(features_train.gencolumns_mut(), self.columns.iter())
		{
			match dataframe_column {
				ColumnView::Number(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data) {
						*a = *b;
					}
				}
				ColumnView::Enum(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data) {
						*a = b.unwrap().get().to_f32().unwrap();
					}
				}
				_ => return None,
			}
		}
		Some(features_train)
	}

	pub fn to_rows(&self) -> Array2<Value<'a>> {
		let mut rows = unsafe { Array2::uninitialized((self.nrows(), self.ncols())) };
		for (mut ndarray_column, dataframe_column) in
			izip!(rows.gencolumns_mut(), self.columns.iter())
		{
			match dataframe_column {
				ColumnView::Unknown(_) => ndarray_column.fill(Value::Unknown),
				ColumnView::Number(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data) {
						*a = Value::Number(*b);
					}
				}
				ColumnView::Enum(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data) {
						*a = Value::Enum(*b);
					}
				}
				ColumnView::Text(column) => {
					for (a, b) in izip!(ndarray_column.iter_mut(), column.data) {
						*a = Value::Text(b);
					}
				}
			}
		}
		rows
	}
}

impl<'a> ColumnView<'a> {
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

	pub fn name(&self) -> &str {
		match self {
			Self::Unknown(s) => s.name,
			Self::Number(s) => s.name,
			Self::Enum(s) => s.name,
			Self::Text(s) => s.name,
		}
	}

	pub fn column_type(&self) -> ColumnTypeView {
		match self {
			Self::Unknown(_) => ColumnTypeView::Unknown,
			Self::Number(_) => ColumnTypeView::Number,
			Self::Enum(column) => ColumnTypeView::Enum {
				options: column.options,
			},
			Self::Text(_) => ColumnTypeView::Text,
		}
	}

	pub fn as_number(&self) -> Option<NumberColumnView> {
		match self {
			Self::Number(s) => Some(s.clone()),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<EnumColumnView> {
		match self {
			Self::Enum(s) => Some(s.clone()),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<TextColumnView> {
		match self {
			Self::Text(s) => Some(s.clone()),
			_ => None,
		}
	}

	pub fn split_at_row(&self, index: usize) -> (Self, Self) {
		match self {
			ColumnView::Unknown(column) => (
				ColumnView::Unknown(UnknownColumnView {
					name: column.name,
					len: index,
				}),
				ColumnView::Unknown(UnknownColumnView {
					name: column.name,
					len: column.len - index,
				}),
			),
			ColumnView::Number(column) => {
				let (data_a, data_b) = column.data.split_at(index);
				(
					ColumnView::Number(NumberColumnView {
						name: column.name,
						data: data_a,
					}),
					ColumnView::Number(NumberColumnView {
						name: column.name,
						data: data_b,
					}),
				)
			}
			ColumnView::Enum(column) => {
				let (data_a, data_b) = column.data.split_at(index);
				(
					ColumnView::Enum(EnumColumnView {
						name: column.name,
						options: column.options,
						data: data_a,
					}),
					ColumnView::Enum(EnumColumnView {
						name: column.name,
						options: column.options,
						data: data_b,
					}),
				)
			}
			ColumnView::Text(column) => {
				let (data_a, data_b) = column.data.split_at(index);
				(
					ColumnView::Text(TextColumnView {
						name: column.name,
						data: data_a,
					}),
					ColumnView::Text(TextColumnView {
						name: column.name,
						data: data_b,
					}),
				)
			}
		}
	}

	pub fn view(&self) -> Self {
		match self {
			ColumnView::Unknown(s) => ColumnView::Unknown(s.view()),
			ColumnView::Number(s) => ColumnView::Number(s.view()),
			ColumnView::Enum(s) => ColumnView::Enum(s.view()),
			ColumnView::Text(s) => ColumnView::Text(s.view()),
		}
	}
}

impl<'a> UnknownColumnView<'a> {
	pub fn view(&self) -> Self {
		self.clone()
	}
}

impl<'a> NumberColumnView<'a> {
	pub fn view(&self) -> Self {
		self.clone()
	}
}

impl<'a> EnumColumnView<'a> {
	pub fn view(&self) -> Self {
		self.clone()
	}
}

impl<'a> TextColumnView<'a> {
	pub fn view(&self) -> Self {
		self.clone()
	}
}

impl<'a> Value<'a> {
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
