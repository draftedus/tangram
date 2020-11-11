use tangram_app_common::date_window::{DateWindow, DateWindowInterval};
use tangram_app_layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub id: String,
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Inner {
	#[serde(rename = "regressor")]
	Regressor(RegressorProductionMetricsOverview),
	#[serde(rename = "binary_classifer")]
	BinaryClassifier(BinaryClassifierProductionMetricsOverview),
	#[serde(rename = "multiclass_classifier")]
	MulticlassClassifier(MulticlassClassifierProductionMetricsOverview),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressorProductionMetricsOverview {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub mse_chart: MSEChart,
	pub overall: RegressionProductionMetrics,
	pub true_values_count_chart: Vec<TrueValuesCountChartEntry>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrueValuesCountChartEntry {
	pub label: String,
	pub count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MSEChart {
	pub data: Vec<MSEChartEntry>,
	pub training_mse: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MSEChartEntry {
	pub label: String,
	pub mse: Option<f32>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressionProductionMetrics {
	pub mse: TrainingProductionMetrics,
	pub rmse: TrainingProductionMetrics,
	pub true_values_count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingProductionMetrics {
	pub production: Option<f32>,
	pub training: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryClassifierProductionMetricsOverview {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub true_values_count_chart: Vec<TrueValuesCountChartEntry>,
	pub overall: BinaryClassificationOverallProductionMetrics,
	pub id: String,
	pub accuracy_chart: AccuracyChart,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticlassClassifierProductionMetricsOverview {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub true_values_count_chart: Vec<TrueValuesCountChartEntry>,
	pub overall: MulticlassClassificationOverallProductionMetrics,
	pub id: String,
	pub accuracy_chart: AccuracyChart,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccuracyChart {
	pub data: Vec<AccuracyChartEntry>,
	pub training_accuracy: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccuracyChartEntry {
	pub accuracy: Option<f32>,
	pub label: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryClassificationOverallProductionMetrics {
	pub accuracy: TrainingProductionMetrics,
	pub precision: TrainingProductionMetrics,
	pub recall: TrainingProductionMetrics,
	pub true_values_count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticlassClassificationOverallProductionMetrics {
	pub accuracy: TrainingProductionMetrics,
	pub class_metrics_table: Vec<ClassMetricsTableEntry>,
	pub true_values_count: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassMetricsTableEntry {
	pub class_name: String,
	pub precision: TrainingProductionMetrics,
	pub recall: TrainingProductionMetrics,
}
