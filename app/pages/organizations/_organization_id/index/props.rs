use tangram_app_common::organizations::Member;
use tangram_app_layouts::app_layout::AppLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub id: String,
	pub members: Vec<Member>,
	pub name: String,
	pub repos: Vec<Repo>,
	pub user_id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	pub id: String,
	pub title: String,
}
