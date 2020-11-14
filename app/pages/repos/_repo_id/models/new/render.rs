use super::props::Props;
use html::html;
use pinwheel::{asset, client};
use tangram_app_common::Context;
use tangram_app_layouts::{
	app_layout::{get_app_layout_info, AppLayout},
	document::PageInfo,
};
use tangram_ui as ui;
use tangram_util::error::Result;

pub async fn render(context: &Context, error: Option<String>) -> Result<String> {
	let app_layout_info = get_app_layout_info(context).await?;
	let props = Props {
		app_layout_info,
		error,
	};
	let tangram = asset!("tangram.png");
	let client_wasm_js_src = client!("client/Cargo.toml");
	let page_info = PageInfo {
		client_wasm_js_src: Some(client_wasm_js_src),
	};
	let html = html! {
		<AppLayout page_info={page_info} _info={props.app_layout_info}>
			<img src={tangram} />
			<ui::Button
				disabled={None}
				download={None}
				href={None}
				id={None}
			>
				{"Click Me"}
			</ui::Button>
		</AppLayout>
	};
	Ok(html.render_to_string())
}
