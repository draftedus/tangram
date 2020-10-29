use crate::{common::predict::PredictionResult, layouts::model_layout::ModelLayoutInfo};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub inner: Inner,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "value")]
pub enum Inner {
	PredictionForm(PredictionForm),
	PredictionResult(PredictionResult),
}

#[derive(serde::Serialize)]
pub struct PredictionForm {
	pub form: PredictForm,
}

#[derive(serde::Serialize)]
pub struct PredictForm {
	pub fields: Vec<Column>,
}

#[derive(serde::Serialize)]
#[serde(tag = "type")]
pub enum Column {
	#[serde(rename = "unknown")]
	Unknown(Unknown),
	#[serde(rename = "number")]
	Number(Number),
	#[serde(rename = "enum")]
	Enum(Enum),
	#[serde(rename = "text")]
	Text(Text),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unknown {
	pub name: String,
	pub value: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Number {
	pub name: String,
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
	pub value: String,
}

#[derive(serde::Serialize)]
pub struct Enum {
	pub name: String,
	pub options: Vec<String>,
	pub value: String,
	pub histogram: Vec<(String, u64)>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Text {
	pub name: String,
	pub value: String,
}
