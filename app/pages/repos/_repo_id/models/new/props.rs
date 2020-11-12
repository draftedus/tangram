use tangram_app_layouts::app_layout::AppLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub error: Option<String>,
}
