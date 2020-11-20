use super::props::Props;
use html::html;
use pinwheel::client;
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
	let client_wasm_js_src = client!();
	let page_info = PageInfo {
		client_wasm_js_src: Some(client_wasm_js_src),
	};
	let html = html! {
		<AppLayout page_info={page_info} info={props.app_layout_info}>
			<ui::S1>
				<ui::H1 center={None}>{"Upload Model"}</ui::H1>
				<ui::Form
					id={None}
					auto_complete={None}
					enc_type={Some("multipart/form-data".to_owned())}
					action={None}
					post={Some(true)}
				>
					{
						props.error.map(|error| {
							html! {
								<ui::Alert
									title={None}
									level={ui::Level::Danger}
								>
									{error}
								</ui::Alert>
							}
						})
					}
					<ui::FileField
						disabled={None}
						label={Some("File".to_owned())}
						name={Some("file".to_owned())}
						required={Some(true)}
					/>
					<ui::Button
						disabled={None}
						download={None}
						href={None}
						id={None}
						button_type={ui::ButtonType::Submit}
					>
						{"Upload"}
					</ui::Button>
				</ui::Form>
			</ui::S1>
		</AppLayout>
	};
	Ok(html.render_to_string())
}
