use tangram_app_layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub id: String,
	pub metrics: TrainedModelMetrics,
	pub model_comparison_metric_name: String,
	pub hyperparameters: Vec<(String, String)>,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(Clone, serde::Serialize)]
pub struct TrainedModelMetrics {
	pub identifier: String,
	pub model_comparison_metric_value: f32,
	pub model_type: String,
	pub time: String,
}
