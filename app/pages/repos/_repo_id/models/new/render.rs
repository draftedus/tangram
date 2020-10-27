use super::props::props;
use crate::Context;
use anyhow::Result;

pub async fn render(context: &Context, error: Option<String>) -> Result<String> {
	let props = props(context, error).await?;
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/new", props)?;
	Ok(html)
}
