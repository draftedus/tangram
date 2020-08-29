#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct StatsSettings {
	pub text_histogram_max_size: u64,
	pub number_histogram_max_size: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum ColumnStats {
	Unknown(UnknownColumnStats),
	Number(NumberColumnStats),
	Enum(EnumColumnStats),
	Text(TextColumnStats),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct UnknownColumnStats {
	pub column_name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct NumberColumnStats {
	pub column_name: String,
	pub invalid_count: u64,
	pub unique_count: u64,
	pub histogram: Option<Vec<(f32, u64)>>,
	pub min: f32,
	pub max: f32,
	pub mean: f32,
	pub variance: f32,
	pub std: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct EnumColumnStats {
	pub column_name: String,
	pub invalid_count: u64,
	pub histogram: Vec<(String, u64)>,
	pub unique_count: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct TextColumnStats {
	pub column_name: String,
	pub top_tokens: Vec<(String, u64)>,
}
