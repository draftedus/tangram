use html::html;
use tangram_app_common::Context;
use tangram_ui as ui;
use tangram_util::error::Result;

pub async fn render(_context: &Context, _error: Option<String>) -> Result<String> {
	// let app_layout_info = get_app_layout_info(context).await?;
	// let props = Props {
	// 	app_layout_info,
	// 	error,
	// };
	// let html = context
	// 	.pinwheel
	// 	.render_with("/repos/_repo_id/models/new", "Hello")?;
	let html = html! {
		<div>
			<ui::Button
				disabled={None}
				download={None}
				href={None}
				id={None}
			>
				{"Click Me!!!"}
			</ui::Button>
		</div>
	};
	Ok(html.render_to_string())
}
