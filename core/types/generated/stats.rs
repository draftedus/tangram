#![allow(clippy::all)]

use buffy::prelude::*;

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct StatsSettings {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub text_histogram_max_size: buffy::Field<u64>,
	#[buffy(id = 2)]
	pub number_histogram_max_size: buffy::Field<u64>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, PartialEq)]
pub enum ColumnStats {
	#[buffy(unknown)]
	UnknownVariant(u64, buffy::WireType, Vec<u8>),
	#[buffy(id = 1)]
	Unknown(UnknownColumnStats),
	#[buffy(id = 2)]
	Number(NumberColumnStats),
	#[buffy(id = 3)]
	Enum(EnumColumnStats),
	#[buffy(id = 4)]
	Text(TextColumnStats),
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct UnknownColumnStats {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub column_name: buffy::Field<String>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct NumberColumnStats {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub column_name: buffy::Field<String>,
	#[buffy(id = 2)]
	pub invalid_count: buffy::Field<u64>,
	#[buffy(id = 3)]
	pub unique_count: buffy::Field<u64>,
	#[buffy(id = 4)]
	pub histogram: buffy::Field<Option<Vec<(f32, u64)>>>,
	#[buffy(id = 5)]
	pub min: buffy::Field<f32>,
	#[buffy(id = 6)]
	pub max: buffy::Field<f32>,
	#[buffy(id = 7)]
	pub mean: buffy::Field<f32>,
	#[buffy(id = 8)]
	pub variance: buffy::Field<f32>,
	#[buffy(id = 9)]
	pub std: buffy::Field<f32>,
	#[buffy(id = 10)]
	pub p25: buffy::Field<f32>,
	#[buffy(id = 11)]
	pub p50: buffy::Field<f32>,
	#[buffy(id = 12)]
	pub p75: buffy::Field<f32>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct EnumColumnStats {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub column_name: buffy::Field<String>,
	#[buffy(id = 2)]
	pub invalid_count: buffy::Field<u64>,
	#[buffy(id = 3)]
	pub histogram: buffy::Field<Vec<(String, u64)>>,
	#[buffy(id = 4)]
	pub unique_count: buffy::Field<u64>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct TextColumnStats {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub column_name: buffy::Field<String>,
	#[buffy(id = 2)]
	pub top_tokens: buffy::Field<Vec<(String, u64)>>,
}
