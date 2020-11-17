use super::props::Props;
use html::html;
use tangram_app_layouts::{app_layout::AppLayout, document::PageInfo};
use tangram_ui as ui;
use tangram_util::error::Result;

// macro_rules! h {
// 	($component:expr, $children:expr) => {
// 		html::Node::Component(html::ComponentNode::Unrendered {
// 			component: Some(Box::new($component)),
// 			children: Some($children),
// 			})
// 	};
// }

// macro_rules! div {
// 	() => {
// 		html::Node::Host(html::HostNode {
// 			name: "div",
// 			attributes: Vec::new(),
// 			children: Vec::new(),
// 			self_closing: false,
// 			})
// 	};
// }

// pub fn view(props: Props) -> html::Node {
// 	let page_info = PageInfo {
// 		client_wasm_js_src: None,
// 	};
// 	AppLayout {
// 		info: props.app_layout_info,
// 		page_info,
// 		children: vec![
// 			ui::S1 {
// 				children:
// 			}.into()
// 		],
// 	}
// 	// 	vec![h!(
// 	// 		ui::S1 {},
// 	// 		vec![h!(ui::H1 { center: None }, vec!["Create New Repo".to_owned()])]
// 	// 	)]
// 	// )
// }

pub async fn render(props: Props) -> Result<String> {
	let page_info = PageInfo {
		client_wasm_js_src: None,
	};
	let owner = props.owner;
	let html = html! {
		<AppLayout page_info={page_info} info={props.app_layout_info}>
			<ui::S1>
				<ui::H1 center={None}>{"Create New Repo"}</ui::H1>
				<ui::Form
					id={None}
					auto_complete={None}
					enc_type={None}
					action={None}
					post={Some(true)}
				>
					{
						props.error.map(|error| html! {
							<ui::Alert
								title={None}
								level={ui::Level::Danger}
							>
								{error}
							</ui::Alert>
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
					{props.owners.map(|owners| html! {
						<ui::SelectField
							placeholder={None}
							id={None}
							label={Some("Owner".to_owned())}
							name={Some("owner".to_owned())}
							required={Some(true)}
							value={owner}
							options={owners.into_iter().map(|owner| ui::SelectFieldOption {
								text: owner.title,
								value: owner.value,
							}).collect()}
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
