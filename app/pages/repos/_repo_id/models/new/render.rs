use super::props::Props;
use html::html;
use tangram_app_common::Context;
use tangram_app_layouts::app_layout::{get_app_layout_info, AppLayout};
use tangram_ui as ui;
use tangram_util::error::Result;

pub async fn render(context: &Context, error: Option<String>) -> Result<String> {
	let app_layout_info = get_app_layout_info(context).await?;
	let props = Props {
		app_layout_info,
		error,
	};
	// let html = context
	// 	.pinwheel
	// 	.render_with("/repos/_repo_id/models/new", "Hello")?;
	let html = html! {
		<AppLayout info={props.app_layout_info}>
			<ui::Button
				disabled={None}
				download={None}
				href={None}
				id={None}
			>
				{"Click Me!!!"}
			</ui::Button>
		</AppLayout>
	};
	Ok(html.render_to_string())
}
