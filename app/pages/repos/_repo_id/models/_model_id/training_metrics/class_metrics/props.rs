use crate::layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub class: String,
	pub classes: Vec<String>,
	pub f1_score: f32,
	pub false_negatives: u64,
	pub false_positives: u64,
	pub id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub precision: f32,
	pub recall: f32,
	pub true_negatives: u64,
	pub true_positives: u64,
}
