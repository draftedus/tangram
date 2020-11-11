use tangram_app_common::predict::{InputTable, Prediction};
use tangram_app_layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub model_layout_info: ModelLayoutInfo,
	pub identifier: String,
	pub inner: Inner,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "value")]
pub enum Inner {
	NotFound,
	Found(Found),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Found {
	pub date: String,
	pub input_table: InputTable,
	pub prediction: Prediction,
}
