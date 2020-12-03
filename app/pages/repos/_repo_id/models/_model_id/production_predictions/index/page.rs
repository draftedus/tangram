use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_deps::html::{self, html};
use tangram_ui as ui;

#[derive(Clone)]
pub struct Props {
	pub model_layout_info: ModelLayoutInfo,
	pub prediction_table: Option<PredictionTable>,
	pub pagination: Pagination,
}

#[derive(Clone)]
pub struct PredictionTable {
	pub rows: Vec<PredictionTableRow>,
}

#[derive(Clone)]
pub struct PredictionTableRow {
	pub date: String,
	pub identifier: String,
	pub output: String,
}

#[derive(Clone)]
pub struct Pagination {
	pub after: Option<usize>,
	pub before: Option<usize>,
}

#[derive(Clone)]
pub struct PaginationRange {
	pub start: usize,
	pub end: usize,
	pub total: usize,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let html = html! {
		<ModelLayout
			info={props.model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::ProductionPredictions}
		>
			<ui::S1>
				<ui::H1 center={false}>{"Production Predictions"}</ui::H1>
				{if props.prediction_table.is_none() {
					html! {<ui::P>{"There are no predictions."}</ui::P>}
				} else { html! {
					<>
						<ui::Form
							autocomplete={None}
							enc_type={None}
							id={None}
							action={None}
							post={true}
						>
							<div class={"search-bar-wrapper".to_owned()}>
								<ui::TextField
									value={None}
									readonly={None}
									placeholder={None}
									required={None}
									disabled={None}
									autocomplete={"off".to_owned()}
									label={"Identifier".to_owned()}
									name={"identifier".to_owned()}
								/>
								<ui::Button
									href={None}
									id={None}
									download={None}
									disabled={None}
									color={None}
									button_type={ui::ButtonType::Submit}
								>
									{"Lookup"}
								</ui::Button>
							</div>
						</ui::Form>
						<ui::Table width={"100%".to_owned()}>
							<ui::TableHeader>
								<ui::TableRow color={None}>
									<ui::TableHeaderCell
										expand={None}
										text_align={None}
										color={None}
									>
										{"Identifier"}
									</ui::TableHeaderCell>
									<ui::TableHeaderCell
										expand={None}
										text_align={None}
										color={None}
									>
										{"Date"}
									</ui::TableHeaderCell>
									<ui::TableHeaderCell
										expand={None}
										text_align={None}
										color={None}
									>
										{"Output"}
									</ui::TableHeaderCell>
								</ui::TableRow>
							</ui::TableHeader>
							<ui::TableBody>
							{props.prediction_table.map(|prediction_table| prediction_table.rows.iter().map(|prediction| html! {
								<ui::TableRow color={None}>
									<ui::TableCell color={None} expand={None}>
										<ui::Link
										class={None}
										title={None}
											href={format!("./predictions/{}", prediction.identifier)}>
											{prediction.identifier.clone()}
										</ui::Link>
									</ui::TableCell>
									<ui::TableCell color={None} expand={None}>
									{prediction.date.clone()}
									</ui::TableCell>
									<ui::TableCell color={None} expand={None}>
										{prediction.output.clone()}
									</ui::TableCell>
								</ui::TableRow>
							}).collect::<Vec<_>>())}
							</ui::TableBody>
						</ui::Table>
						<div class="pagination-buttons">
							<ui::Form
								autocomplete={None}
								enc_type={None}
								id={None}
								action={None}
								post={None}
							>
								{props.pagination.after.map(|after| html! {
									<input
										name={"after".to_owned()}
										type={"hidden".to_owned()}
										value={after.to_string()}
									/>
								})}
								<ui::Button
									download={None}
									href={None}
									id={None}
									color={None}
									disabled={props.pagination.after.is_none()}
									button_type={ui::ButtonType::Submit}
								>
									{"Newer"}
								</ui::Button>
							</ui::Form>
							<ui::Form
								autocomplete={None}
								enc_type={None}
								id={None}
								action={None}
								post={None}
							>
								{props.pagination.before.map(|before| html! {
									<input
										name={"before".to_owned()}
										type={"hidden".to_owned()}
										value={before.to_string()}
									/>
								})}
								<ui::Button
									download={None}
									href={None}
									id={None}
									color={None}
									disabled={props.pagination.before.is_none()}
									button_type={ui::ButtonType::Submit}
								>
									{"Older"}
								</ui::Button>
							</ui::Form>
						</div>
					</>
				}
			}}
			</ui::S1>
		</ModelLayout>
	};
	html.render_to_string()
}
