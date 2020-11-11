use tangram_app_layouts::app_layout::AppLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub error: Option<String>,
	pub title: Option<String>,
	pub owner: Option<String>,
	pub owners: Option<Vec<Owner>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
	pub value: String,
	pub title: String,
}
