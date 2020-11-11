use tangram_app_layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub id: String,
	pub roc_curve_data: Vec<ROCCurveData>,
	pub model_layout_info: ModelLayoutInfo,
	pub class: String,
	pub auc_roc: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ROCCurveData {
	pub false_positive_rate: f32,
	pub true_positive_rate: f32,
}
