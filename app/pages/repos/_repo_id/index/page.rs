use html::html;
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::PageInfo,
};
use tangram_ui as ui;

pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub models: Vec<Model>,
	pub title: String,
}

pub struct Model {
	pub id: String,
	pub created_at: String,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let model_list_table = html! {
		<ui::Table width={"100%".to_owned()}>
			<ui::TableHeader>
				<ui::TableRow color={None}>
					<ui::TableHeaderCell
						color={None}
						expand={None}
						text_align={None}
					>
						{"Id"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						expand={None}
						text_align={None}
					>
						{"Created"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						expand={None}
						text_align={None}
					>
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{props.models.iter().map(|model| html! {
					<ui::TableRow color={None}>
						<ui::TableCell color={None} expand={false}>
							<ui::Link
								title={None}
								class={None}
								href={format!("./models/{}/", model.id)}
							>
								{model.id.clone()}
							</ui::Link>
						</ui::TableCell>
						<ui::TableCell
							color={None}
							expand={None}
						>
							{model.created_at.clone()}
						</ui::TableCell>
						<ui::TableCell expand={false} color={None}>
							<form method="post">
								<input
									name="action"
									type="hidden"
									value="delete_model"
								/>
								<input name="model_id" type="hidden" value={model.id.clone()} />
								<ui::Button
									button_type={ui::ButtonType::Button}
									disabled={None}
									download={None}
									href={None}
									id={None}
									color={"var(--red)".to_owned()}
								>
									{"Delete"}
								</ui::Button>
							</form>
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	};

	let html = html! {
		<AppLayout info={props.app_layout_info} page_info={page_info}>
			<ui::S1>
				<ui::SpaceBetween>
					<ui::H1 center={false}>{props.title}</ui::H1>
					<ui::Button
						button_type={ui::ButtonType::Button}
						download={None}
						id={None}
						disabled={None}
						color={None}
						href={"./models/new".to_owned()}
					>
						{"Upload New Version"}
					</ui::Button>
				</ui::SpaceBetween>
				<ui::S2>
				{if !props.models.is_empty() {
					model_list_table
				} else {
					html! {
						<ui::Card>
							<ui::P>{"This repositories has no models."}</ui::P>
						</ui::Card>
					}
				}}
				</ui::S2>
			</ui::S1>
		</AppLayout>
	};

	html.render_to_string()
}
