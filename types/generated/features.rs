#![allow(clippy::all)]

use buffy::prelude::*;

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, PartialEq)]
pub enum FeatureGroup {
	#[buffy(unknown)]
	UnknownVariant(u64, buffy::WireType, Vec<u8>),
	#[buffy(id = 1)]
	Identity(IdentityFeatureGroup),
	#[buffy(id = 2)]
	Normalized(NormalizedFeatureGroup),
	#[buffy(id = 3)]
	OneHotEncoded(OneHotEncodedFeatureGroup),
	#[buffy(id = 4)]
	BagOfWords(BagOfWordsFeatureGroup),
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct IdentityFeatureGroup {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub source_column_name: buffy::Field<String>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct NormalizedFeatureGroup {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub source_column_name: buffy::Field<String>,
	#[buffy(id = 2)]
	pub mean: buffy::Field<f32>,
	#[buffy(id = 3)]
	pub variance: buffy::Field<f32>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct OneHotEncodedFeatureGroup {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub source_column_name: buffy::Field<String>,
	#[buffy(id = 2)]
	pub categories: buffy::Field<Vec<String>>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct BagOfWordsFeatureGroup {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub source_column_name: buffy::Field<String>,
	#[buffy(id = 2)]
	pub tokenizer: buffy::Field<Tokenizer>,
	#[buffy(id = 3)]
	pub tokens: buffy::Field<Vec<(String, f32)>>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, PartialEq)]
pub enum Tokenizer {
	#[buffy(unknown)]
	UnknownVariant(u64, buffy::WireType, Vec<u8>),
	#[buffy(id = 1)]
	Alphanumeric,
}
