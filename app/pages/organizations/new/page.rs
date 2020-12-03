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
		<AppLayout info={props.app_layout_info} page_info={page_info}>
			<ui::S1>
				<ui::H1 center={false}>{"Create New Organization"}</ui::H1>
				<ui::Form
					id={None}
					autocomplete={None}
					action={None}
					post={true}
					enc_type={None}
				>
					<ui::TextField
						value={None}
						placeholder={None}
						readonly={None}
						autocomplete={None}
						disabled={None}
						label={"Name".to_owned()}
						name={"name".to_owned()}
						required={true}
					/>
					<ui::Button
						button_type={ui::ButtonType::Submit}
						disabled={None}
						download={None}
						href={None}
						id={None}
						color={None}
					>
						{"Create"}
					</ui::Button>
				</ui::Form>
			</ui::S1>
		</AppLayout>
	};
	html.render_to_string()
}
