use crate::layouts::app_layout::AppLayoutInfo;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub repos: Vec<Repo>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
	pub id: String,
	pub title: String,
	pub created_at: String,
	pub owner_name: Option<String>,
}
