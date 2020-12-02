use html::html;
use tangram_app_layouts::app_layout::{AppLayout, AppLayoutInfo};
use tangram_app_layouts::document::PageInfo;
use tangram_ui as ui;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub error: Option<String>,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let html = html! {
		<AppLayout info={props.app_layout_info} page_info={page_info}>
			<ui::S1>
			<ui::H1 center={Some(true)}>{"Edit Organization"}</ui::H1>
			<ui::Form
				autocomplete={None}
				enc_type={None}
				action={None}
				post={Some(true)}
				id={None}
			>
				<ui::TextField
					autocomplete={None}
					disabled={None}
					placeholder={None}
					readonly={None}
					required={None}
					value={None}
					label={Some("Organization Name".to_owned())}
					name={Some("name".to_owned())}
				/>
				<ui::Button
					id={None}
					color={None}
					disabled={None}
					href={None}
					download={None}
					button_type={ui::ButtonType::Submit}
				>
					{"Submit"}
				</ui::Button>
			</ui::Form>
			</ui::S1>
		</AppLayout>
	};
	html.render_to_string()
}
