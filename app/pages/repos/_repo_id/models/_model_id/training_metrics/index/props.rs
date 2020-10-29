use crate::layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub id: String,
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "value")]
pub enum Inner {
	Regressor(RegressorProps),
	BinaryClassifier(BinaryClassifierProps),
	MulticlassClassifier(MulticlassClassifierProps),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressorProps {
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
	pub mse: f32,
	pub rmse: f32,
	pub id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryClassifierProps {
	pub accuracy: f32,
	pub baseline_accuracy: f32,
	pub auc_roc: f32,
	pub id: String,
	pub precision: f32,
	pub recall: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticlassClassifierProps {
	pub accuracy: f32,
	pub baseline_accuracy: f32,
	pub class_metrics: Vec<ClassMetrics>,
	pub classes: Vec<String>,
	pub id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassMetrics {
	pub precision: f32,
	pub recall: f32,
}
