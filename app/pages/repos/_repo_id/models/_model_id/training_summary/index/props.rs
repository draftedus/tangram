use tangram_app_layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub id: String,
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
pub struct Inner {
	pub num_models: usize,
	pub total_training_time: String,
	pub trained_models: Vec<TrainedModel>,
	pub winning_model_hyperparameters: Vec<(String, String)>,
}

#[derive(serde::Serialize, Clone)]
pub struct TrainedModel {
	pub identifier: String,
	pub metric: f32,
	pub model_type: String,
	pub time: String,
}
