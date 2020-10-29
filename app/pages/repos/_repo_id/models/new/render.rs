use super::props::*;
use crate::{layouts::app_layout::get_app_layout_info, Context};
use anyhow::Result;

pub async fn render(context: &Context, error: Option<String>) -> Result<String> {
	let props = props(context, error).await?;
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/new", props)?;
	Ok(html)
}

pub async fn props(context: &Context, error: Option<String>) -> Result<Props> {
	let app_layout_info = get_app_layout_info(context).await?;
	Ok(Props {
		app_layout_info,
		error,
	})
}
