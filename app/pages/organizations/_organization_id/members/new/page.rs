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
				<ui::H1 center={false}>
				<ui::Form
					action={None}
					autocomplete={None}
					enc_type={None}
					id={None}
					post={true}
				>
					<ui::TextField
						autocomplete={None}
						disabled={None}
						label={"Email".to_owned()}
						name={"email".to_owned()}
						placeholder={None}
						readonly={None}
						required={None}
						value={None}
					/>
					<ui::CheckboxField
						placeholder={None}
						readonly={None}
						value={None}
						label={"Admin".to_owned()}
						name={"isAdmin".to_owned()}
					/>
					<ui::Button
						disabled={None}
						download={None}
						href={None}
						id={None}
						button_type={ui::ButtonType::Submit}
						color={None}
					>
						{"Invite"}
					</ui::Button>
				</ui::Form>
				</ui::H1>
			</ui::S1>
		</AppLayout>
	};
	html.render_to_string()
}
