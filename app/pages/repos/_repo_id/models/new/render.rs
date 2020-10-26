use super::props::props;
use crate::Context;
use anyhow::Result;

pub struct Options {
	pub error: String,
}

pub async fn render(context: &Context, options: Option<Options>) -> Result<String> {
	let props = props(context, options.as_ref().map(|o| o.error.to_owned())).await?;
	let html = context
		.pinwheel
		.render_with("/repos/_repo_id/models/new", props)?;
	Ok(html)
}
