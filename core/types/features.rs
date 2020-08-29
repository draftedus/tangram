#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum FeatureGroup {
	Identity(IdentityFeatureGroup),
	Normalized(NormalizedFeatureGroup),
	OneHotEncoded(OneHotEncodedFeatureGroup),
	BagOfWords(BagOfWordsFeatureGroup),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct IdentityFeatureGroup {
	pub source_column_name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct NormalizedFeatureGroup {
	pub source_column_name: String,
	pub mean: f32,
	pub variance: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct OneHotEncodedFeatureGroup {
	pub source_column_name: String,
	pub categories: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct BagOfWordsFeatureGroup {
	pub source_column_name: String,
	pub tokenizer: Tokenizer,
	pub tokens: Vec<(String, f32)>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum Tokenizer {
	Alphanumeric,
}
