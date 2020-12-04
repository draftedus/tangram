use tangram_app_common::date_window::DateWindow;
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_deps::html::{self, html};
use tangram_ui as ui;

pub use crate::enum_column::*;
pub use crate::number_column::*;
pub use crate::text_column::*;

pub struct Props {
	pub date_window: DateWindow,
	pub column_name: String,
	pub id: String,
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(Clone)]
pub enum Inner {
	Number(NumberColumnProps),
	Enum(EnumColumnProps),
	Text(TextColumnProps),
}

#[derive(Clone)]
pub struct IntervalBoxChartDataPoint {
	pub label: String,
	pub stats: Option<IntervalBoxChartDataPointStats>,
}

#[derive(Clone)]
pub struct IntervalBoxChartDataPointStats {
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

#[derive(Clone)]
pub struct OverallBoxChartData {
	pub production: Option<OverallBoxChartDataStats>,
	pub training: OverallBoxChartDataStats,
}

#[derive(Clone)]
pub struct OverallBoxChartDataStats {
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let inner = match props.inner {
		Inner::Number(inner) => html! {<NumberColumn props={inner} />},
		Inner::Enum(inner) => html! {<EnumColumn props={inner} />},
		Inner::Text(inner) => html! {<TextColumn props={inner} />},
	};
	let html = html! {
		<ModelLayout
			info={props.model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::ProductionStats}
		>
			<ui::S1>
				<ui::H1 center={false}>{props.column_name}</ui::H1>
				<ui::Form
					post={None}
					id={None}
					enc_type={None}
					autocomplete={None}
					action={None}
				>
					// <DateWindowSelectField dateWindow={props.dateWindow} />
					<noscript>
						<ui::Button
							disabled={None}
							download={None}
							href={None}
							id={None}
							button_type={ui::ButtonType::Submit}
							color={None}
						>
							{"Submit"}
						</ui::Button>
					</noscript>
				</ui::Form>
				{inner}
			</ui::S1>
		</ModelLayout>
	};
	html.render_to_string()
}
