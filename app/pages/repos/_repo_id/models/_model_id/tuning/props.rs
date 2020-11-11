use tangram_app_layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub tuning: Option<Inner>,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inner {
	pub baseline_threshold: f32,
	pub metrics: Vec<Metrics>,
	pub class: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
	pub accuracy: f32,
	pub f1_score: f32,
	pub false_negatives: u64,
	pub false_positives: u64,
	pub precision: f32,
	pub recall: f32,
	pub threshold: f32,
	pub true_negatives: u64,
	pub true_positives: u64,
}
