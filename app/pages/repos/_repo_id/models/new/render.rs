use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::PageInfo,
};
use tangram_deps::html::{self, html};
use tangram_ui as ui;

pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub error: Option<String>,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let html = html! {
		<AppLayout page_info={page_info} info={props.app_layout_info}>
			<ui::S1>
				<ui::H1 center={None}>{"Upload Model"}</ui::H1>
				<ui::Form
					id={None}
					autocomplete={None}
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
						color={None}
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
	html.render_to_string()
}
