use tangram_app_layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub num_models: usize,
	pub total_training_time: String,
	pub trained_models_metrics: Vec<TrainedModel>,
	pub best_model_metrics: TrainedModel,
	pub model_comparison_metric_name: String,
	pub best_model_hyperparameters: Vec<(String, String)>,
}

#[derive(serde::Serialize, Clone)]
pub struct TrainedModel {
	pub identifier: String,
	pub model_comparison_metric_value: f32,
	pub model_type: String,
	pub time: String,
}
