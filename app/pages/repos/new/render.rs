use super::props::Props;
use html::html;
use tangram_app_layouts::{app_layout::AppLayout, document::PageInfo};
use tangram_ui as ui;
use tangram_util::error::Result;

pub async fn render(props: Props) -> Result<String> {
	let page_info = PageInfo {
		client_wasm_js_src: None,
	};
	let html = html! {
		<AppLayout page_info={page_info} info={props.app_layout_info}>
			<ui::S1>
				<ui::H1 center={None}>{"Create New Repo Rust"}</ui::H1>
				<ui::Form
					id={None}
					auto_complete={None}
					enc_type={None}
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
					<ui::TextField
						label={Some("Title".to_owned())}
						name={Some("title".to_owned())}
						value={props.title}
						autocomplete={None}
						read_only={None}
						disabled={None}
						required={None}
						placeholder={None}
					/>
					{props.owner.map(|owner| html! {
						<ui::SelectField
							placeholder={None}
							id={None}
							label={Some("Owner".to_owned())}
							name={Some("owner".to_owned())}
							required={Some(true)}
							value={Some(owner)}
							options={None}
							disabled={None}
						/>
					})}
					<ui::Button
						disabled={None}
						download={None}
						href={None}
						id={None}
						button_type={ui::ButtonType::Submit}
					>
						{"Submit"}
					</ui::Button>
				</ui::Form>
			</ui::S1>
		</AppLayout>
	};
	Ok(html.render_to_string())
}
