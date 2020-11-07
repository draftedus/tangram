use crate::{common::repos::Repo, layouts::app_layout::AppLayoutInfo};

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub repos: Vec<Repo>,
}
