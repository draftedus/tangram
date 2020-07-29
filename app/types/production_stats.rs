use chrono::prelude::*;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductionStatsResponse {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: ProductionStats,
	pub intervals: Vec<ProductionStats>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
pub enum DateWindow {
	#[serde(rename = "today")]
	Today,
	#[serde(rename = "this_month")]
	ThisMonth,
	#[serde(rename = "this_year")]
	ThisYear,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub enum DateWindowInterval {
	Hourly,
	Daily,
	Monthly,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductionStats {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub predictions_count: u64,
	pub column_stats: Vec<ProductionColumnStats>,
	pub prediction_stats: ProductionPredictionStats,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum ProductionColumnStats {
	Unknown(UnknownProductionColumnStats),
	Text(TextProductionColumnStats),
	Number(NumberProductionColumnStats),
	Enum(EnumProductionColumnStats),
}

impl ProductionColumnStats {
	pub fn column_name(&self) -> &str {
		match &self {
			Self::Unknown(c) => c.column_name.as_str(),
			Self::Text(c) => c.column_name.as_str(),
			Self::Number(c) => c.column_name.as_str(),
			Self::Enum(c) => c.column_name.as_str(),
		}
	}
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UnknownProductionColumnStats {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub alert: Option<String>,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberProductionColumnStats {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub stats: Option<NumberStats>,
	pub alert: Option<String>,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberStats {
	pub n: u64,
	pub min: f32,
	pub max: f32,
	pub mean: f32,
	pub variance: f32,
	pub std: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnumProductionColumnStats {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub histogram: Vec<(String, u64)>,
	pub invalid_histogram: Option<Vec<(String, u64)>>,
	pub alert: Option<String>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TextProductionColumnStats {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub alert: Option<String>,
	pub token_histogram: Vec<(String, u64)>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum ProductionPredictionStats {
	Regression(RegressionProductionPredictionStats),
	Classification(ClassificationProductionPredictionStats),
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegressionProductionPredictionStats {
	pub stats: Option<NumberStats>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationProductionPredictionStats {
	pub histogram: Vec<(String, u64)>,
}

pub enum ProductionStatsSingleColumnResponse {
	Number(NumberProductionStatsSingleColumnResponse),
	Enum(EnumProductionStatsSingleColumnResponse),
	Text(TextProductionStatsSingleColumnResponse),
}

pub struct NumberProductionStatsSingleColumnResponse {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: NumberProductionStatsSingleColumn,
	pub intervals: Vec<NumberProductionStatsSingleColumn>,
}

pub struct NumberProductionStatsSingleColumn {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub predictions_count: u64,
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub stats: Option<NumberStats>,
	pub alert: Option<String>,
}

pub struct EnumProductionStatsSingleColumnResponse {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: EnumProductionStatsSingleColumn,
	pub intervals: Vec<EnumProductionStatsSingleColumn>,
}

pub struct EnumProductionStatsSingleColumn {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub predictions_count: u64,
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub histogram: Vec<(String, u64)>,
	pub invalid_histogram: Option<Vec<(String, u64)>>,
	pub alert: Option<String>,
}

pub struct TextProductionStatsSingleColumnResponse {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: TextProductionStatsSingleColumn,
}

pub struct TextProductionStatsSingleColumn {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub predictions_count: u64,
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub token_histogram: Vec<(String, u64)>,
	pub alert: Option<String>,
}
