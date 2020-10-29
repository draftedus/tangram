use crate::layouts::app_layout::AppLayoutInfo;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub models: Vec<Model>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Model {
	pub id: String,
	pub created_at: String,
}
