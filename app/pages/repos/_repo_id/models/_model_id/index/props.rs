use crate::layouts::model_layout::ModelLayoutInfo;

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
	Regressor(RegressorProps),
	#[serde(rename = "binary_classifier")]
	BinaryClassifier(BinaryClassifierProps),
	#[serde(rename = "multiclass_classifier")]
	MulticlassClassifier(MulticlassClassifierProps),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressorProps {
	pub id: String,
	pub metrics: RegressorInnerMetrics,
	pub training_summary: TrainingSummary,
	pub losses_chart_data: Option<Vec<f32>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressorInnerMetrics {
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
	pub mse: f32,
	pub rmse: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryClassifierProps {
	pub id: String,
	pub metrics: BinaryClassifierInnerMetrics,
	pub training_summary: TrainingSummary,
	pub losses_chart_data: Option<Vec<f32>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryClassifierInnerMetrics {
	pub baseline_accuracy: f32,
	pub auc_roc: f32,
	pub accuracy: f32,
	pub precision: f32,
	pub recall: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticlassClassifierProps {
	pub id: String,
	pub metrics: MulticlassClassifierInnerMetrics,
	pub training_summary: TrainingSummary,
	pub losses_chart_data: Option<Vec<f32>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticlassClassifierInnerMetrics {
	pub accuracy: f32,
	pub baseline_accuracy: f32,
	pub class_metrics: Vec<MulticlassClassifierInnerClassMetrics>,
	pub classes: Vec<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticlassClassifierInnerClassMetrics {
	pub precision: f32,
	pub recall: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingSummary {
	pub chosen_model_type_name: String,
	pub column_count: usize,
	pub model_comparison_metric_type_name: String,
	pub train_row_count: usize,
	pub test_row_count: usize,
}
