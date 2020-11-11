use tangram_app_common::date_window::{DateWindow, DateWindowInterval};
use tangram_app_layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub date_window: DateWindow,
	pub column_name: String,
	pub id: String,
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum Inner {
	Number(NumberProps),
	Enum(EnumProps),
	Text(TextProps),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NumberProps {
	pub absent_count: u64,
	pub alert: Option<String>,
	pub column_name: String,
	pub date_window_interval: DateWindowInterval,
	pub date_window: DateWindow,
	pub interval_box_chart_data: Vec<IntervalBoxChartDataPoint>,
	pub invalid_count: u64,
	pub max_comparison: NumberTrainingProductionComparison,
	pub mean_comparison: NumberTrainingProductionComparison,
	pub min_comparison: NumberTrainingProductionComparison,
	pub overall_box_chart_data: OverallBoxChartData,
	pub row_count: u64,
	pub std_comparison: NumberTrainingProductionComparison,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntervalBoxChartDataPoint {
	pub label: String,
	pub stats: Option<IntervalBoxChartDataPointStats>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntervalBoxChartDataPointStats {
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverallBoxChartData {
	pub production: Option<OverallBoxChartDataStats>,
	pub training: OverallBoxChartDataStats,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverallBoxChartDataStats {
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NumberTrainingProductionComparison {
	pub production: Option<f32>,
	pub training: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumProps {
	pub alert: Option<String>,
	pub absent_count: u64,
	pub column_name: String,
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub invalid_count: u64,
	pub overall_chart_data: Vec<(String, EnumOverallHistogramEntry)>,
	pub overall_invalid_chart_data: Option<Vec<(String, u64)>>,
	pub row_count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumIntervalChartDataPoint {
	pub label: String,
	pub histogram: Vec<(String, u64)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumOverallHistogramEntry {
	pub production_count: u64,
	pub production_fraction: f32,
	pub training_count: u64,
	pub training_fraction: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextProps {
	pub absent_count: u64,
	pub alert: Option<String>,
	pub column_name: String,
	pub date_window_interval: DateWindowInterval,
	pub date_window: DateWindow,
	pub invalid_count: u64,
	pub overall_token_histogram: Vec<(String, u64)>,
	pub row_count: u64,
}
