use html::html;
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::PageInfo,
};
use tangram_ui as ui;

pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub error: Option<String>,
	pub title: Option<String>,
	pub owner: Option<String>,
	pub owners: Option<Vec<Owner>>,
}

pub struct Owner {
	pub value: String,
	pub title: String,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let owner = props.owner;
	let html = html! {
		<AppLayout info={props.app_layout_info} page_info={page_info}>
			<ui::S1>
				<ui::H1 center={false}>{"Create New Repo"}</ui::H1>
				<ui::Form
					action={None}
					autocomplete={None}
					enc_type={None}
					id={None}
					post={true}
				>
					{props.error.map(|error| html! {
						<ui::Alert
							title={None}
							level={ui::Level::Danger}
						>
							{error}
						</ui::Alert>
					})}
					<ui::TextField
						readonly={false}
						autocomplete={None}
						disabled={None}
						placeholder={None}
						label={"Title".to_owned()}
						name={"title".to_owned()}
						required={true}
						value={props.title}
					/>
					{props.owners.map(|owners| html! {
						<ui::SelectField
							disabled={None}
							id={None}
							placeholder={None}
							label={"Owner".to_owned()}
							name={"owner".to_owned()}
							options={owners.into_iter().map(|owner| ui::SelectFieldOption {
								text: owner.title,
								value: owner.value,
							}).collect::<Vec<_>>()}
							required={true}
							value={owner}
						/>
					})}
					<ui::Button
						id={None}
						color={None}
						disabled={None}
						download={None}
						href={None}
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
