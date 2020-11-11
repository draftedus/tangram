use tangram_app_common::repos::Repo;
use tangram_app_layouts::app_layout::AppLayoutInfo;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub repos: Vec<Repo>,
}
