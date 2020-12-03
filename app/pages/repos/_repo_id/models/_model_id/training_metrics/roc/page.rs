use html::html;
use num_traits::ToPrimitive;
use tangram_app_common::definitions::{AUC_ROC, RECEIVER_OPERATING_CHARACTERISTIC};
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_charts::{
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_ui as ui;

pub struct Props {
	pub id: String,
	pub roc_curve_data: Vec<ROCCurveData>,
	pub model_layout_info: ModelLayoutInfo,
	pub class: String,
	pub auc_roc: f32,
}

pub struct ROCCurveData {
	pub false_positive_rate: f32,
	pub true_positive_rate: f32,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let roc_series = props
		.roc_curve_data
		.iter()
		.map(|roc_curve_series| LineChartPoint {
			x: roc_curve_series.false_positive_rate.to_f64().unwrap(),
			y: roc_curve_series.true_positive_rate.to_f64().unwrap(),
		})
		.collect::<Vec<_>>();
	let roc_series = vec![
		LineChartSeries {
			color: ui::colors::BLUE.to_owned(),
			data: roc_series,
			line_style: Some(LineStyle::Solid),
			point_style: Some(PointStyle::Circle),
			title: Some("ROC".to_owned()),
		},
		LineChartSeries {
			color: ui::colors::BLUE.to_owned(),
			data: vec![
				LineChartPoint { x: 0.0, y: 0.0 },
				LineChartPoint { x: 1.0, y: 1.0 },
			],
			line_style: Some(LineStyle::Dashed),
			point_style: Some(PointStyle::Hidden),
			title: Some("Reference".to_owned()),
		},
	];
	let html = html! {
		<ModelLayout
			info={props.model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::TrainingMetrics}
		>
			<ui::S1>
				<ui::H1 center={false}>{"Training Metrics"}</ui::H1>
				<ui::TabBar>
					<ui::TabLink
						disabled={false}
						href={"./".to_owned()}
						selected={false}
					>
						{"Overview"}
					</ui::TabLink>
					<ui::TabLink
						disabled={false}
						href={"precision_recall".to_owned()}
						selected={false}
					>
						{"PR Curve"}
					</ui::TabLink>
					<ui::TabLink
						disabled={false}
						href={"roc".to_owned()}
						selected={false}
					>
						{"ROC Curve"}
					</ui::TabLink>
				</ui::TabBar>
				<ui::S2>
					<ui::H2 center={false}>{"Area Under the Receiver Operating Characteristic"}</ui::H2>
					<ui::P>{AUC_ROC}</ui::P>
					<ui::Card>
						<ui::NumberChart
							title="AUC"
							value={ui::format_number(props.auc_roc)}
						/>
					</ui::Card>
				</ui::S2>
				<ui::S2>
					<ui::H2 center={false}>{"Receiver Operating Characteristic Curve"}</ui::H2>
					<ui::P>{RECEIVER_OPERATING_CHARACTERISTIC}</ui::P>
					<ui::Card>
						<LineChart
							class={None}
							labels={None}
							should_draw_x_axis_labels={None}
							should_draw_y_axis_labels={None}
							x_axis_grid_line_interval={None}
							y_axis_grid_line_interval={None}
							hide_legend={false}
							id={"roc".to_owned()}
							series={roc_series}
							title={"Receiver Operating Characteristic Curve".to_owned()}
							x_axis_title={"False Positive Rate".to_owned()}
							x_max={1.0}
							x_min={0.0}
							y_axis_title={"True Positive Rate".to_owned()}
							y_max={1.0}
							y_min={0.0}
						/>
					</ui::Card>
				</ui::S2>
			</ui::S1>
		</ModelLayout>
	};
	html.render_to_string()
}
