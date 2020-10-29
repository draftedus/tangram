use crate::{
	common::date_window::{DateWindow, DateWindowInterval},
	layouts::model_layout::ModelLayoutInfo,
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub model_id: String,
	pub overall_column_stats_table: Vec<OverallColumnStats>,
	pub prediction_count_chart: Vec<PredictionCountChartEntry>,
	pub prediction_stats_chart: PredictionStatsChart,
	pub prediction_stats_interval_chart: PredictionStatsIntervalChart,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverallColumnStats {
	pub absent_count: u64,
	pub invalid_count: u64,
	pub alert: Option<String>,
	pub name: String,
	pub column_type: ColumnType,
}

#[derive(serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum PredictionStatsChart {
	#[serde(rename = "regression")]
	Regression(RegressionChartEntry),
	#[serde(rename = "binary_classification")]
	BinaryClassification(BinaryClassificationChartEntry),
	#[serde(rename = "multiclass_classification")]
	MulticlassClassification(MulticlassClassificationChartEntry),
}

#[derive(serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum PredictionStatsIntervalChart {
	#[serde(rename = "regression")]
	Regression(Vec<RegressionChartEntry>),
	#[serde(rename = "classification")]
	BinaryClassification(Vec<BinaryClassificationChartEntry>),
	#[serde(rename = "classification")]
	MulticlassClassification(Vec<MulticlassClassificationChartEntry>),
}

#[derive(serde::Serialize)]
pub enum ColumnType {
	#[serde(rename = "unknown")]
	Unknown,
	#[serde(rename = "number")]
	Number,
	#[serde(rename = "enum")]
	Enum,
	#[serde(rename = "text")]
	Text,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionCountChartEntry {
	pub count: u64,
	pub label: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressionChartEntry {
	pub label: String,
	pub quantiles: ProductionTrainingQuantiles,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryClassificationChartEntry {
	pub label: String,
	pub histogram: ProductionTrainingHistogram,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticlassClassificationChartEntry {
	pub label: String,
	pub histogram: ProductionTrainingHistogram,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionTrainingHistogram {
	pub production: Vec<(String, u64)>,
	pub training: Vec<(String, u64)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionTrainingQuantiles {
	pub production: Option<Quantiles>,
	pub training: Quantiles,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Quantiles {
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}
