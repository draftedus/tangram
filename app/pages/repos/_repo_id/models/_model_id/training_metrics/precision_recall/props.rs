use tangram_app_layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub class: String,
	pub precision_recall_curve_data: Vec<PrecisionRecallPoint>,
	pub id: String,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrecisionRecallPoint {
	pub precision: f32,
	pub recall: f32,
	pub threshold: f32,
}
