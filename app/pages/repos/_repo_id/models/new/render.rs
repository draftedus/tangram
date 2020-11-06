use super::props::Props;
use crate::{layouts::app_layout::get_app_layout_info, Context};
use tangram_util::error::Result;

pub async fn render(context: &Context, error: Option<String>) -> Result<String> {
	let app_layout_info = get_app_layout_info(context).await?;
	let props = Props {
		app_layout_info,
		error,
	};
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/new", props)?;
	Ok(html)
}
