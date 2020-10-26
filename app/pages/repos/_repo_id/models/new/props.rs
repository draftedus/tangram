use crate::{
	layouts::app_layout::{get_app_layout_info, AppLayoutInfo},
	Context,
};
use anyhow::Result;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub error: Option<String>,
}

pub async fn props(context: &Context, error: Option<String>) -> Result<Props> {
	let app_layout_info = get_app_layout_info(context).await?;
	Ok(Props {
		app_layout_info,
		error,
	})
}
