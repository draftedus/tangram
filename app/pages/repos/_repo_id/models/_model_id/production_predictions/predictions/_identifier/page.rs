use tangram_app_common::predict::{
	InputTable, Prediction, PredictionResult, PredictionResultProps,
};
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_deps::html::{self, html};
use tangram_ui as ui;

#[derive(Clone)]
pub struct Props {
	pub model_layout_info: ModelLayoutInfo,
	pub identifier: String,
	pub inner: Inner,
}

#[derive(Clone)]
pub enum Inner {
	NotFound,
	Found(Found),
}

#[derive(Clone)]
pub struct Found {
	pub date: String,
	pub input_table: InputTable,
	pub prediction: Prediction,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let inner = match props.inner {
		Inner::NotFound => {
			html! {
				<ui::P>
					<ui::Alert
						title={None}
						level={ui::Level::Danger}
					>
						{"Prediction with identifier "}
						<b>{props.identifier}</b>
						{" not found"}
					</ui::Alert>
				</ui::P>
			}
		}
		Inner::Found(inner) => html! {
			<>
				<ui::Table width={"100%".to_owned()}>
					<ui::TableHeader
					>
						<ui::TableRow
							color={None}
						>
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
						</ui::TableRow>
					</ui::TableHeader>
					<ui::TableBody>
						<ui::TableRow
							color={None}
						>
							<ui::TableCell
								color={None}
								expand={None}
							>
								{props.identifier}
							</ui::TableCell>
							<ui::TableCell
								color={None}
								expand={None}
							>
								{inner.date}
							</ui::TableCell>
						</ui::TableRow>
					</ui::TableBody>
				</ui::Table>
				<PredictionResult
					props={
						PredictionResultProps {
							prediction: inner.prediction,
							input_table: inner.input_table
						}
					}
				/>
			</>
		},
	};
	let html = html! {
		<ModelLayout
			info={props.model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::ProductionPredictions}
		>
			<ui::S1>
				<ui::H1 center={false}>{"Prediction"}</ui::H1>
				{inner}
			</ui::S1>
		</ModelLayout>
	};
	html.render_to_string()
}
