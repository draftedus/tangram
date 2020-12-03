use tangram_app_common::definitions::PRECISION_RECALL;
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_charts::{
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_deps::html::{self, html};
use tangram_deps::num_traits::ToPrimitive;
use tangram_ui as ui;

pub struct Props {
	pub class: String,
	pub precision_recall_curve_series: Vec<PrecisionRecallPoint>,
	pub id: String,
	pub model_layout_info: ModelLayoutInfo,
}

pub struct PrecisionRecallPoint {
	pub precision: f32,
	pub recall: f32,
	pub threshold: f32,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let pr_series = props
		.precision_recall_curve_series
		.iter()
		.map(|threshold| LineChartPoint {
			x: threshold.recall.to_f64().unwrap(),
			y: threshold.precision.to_f64().unwrap(),
		})
		.collect::<Vec<_>>();
	let precision_series = props
		.precision_recall_curve_series
		.iter()
		.map(|threshold| LineChartPoint {
			x: threshold.threshold.to_f64().unwrap(),
			y: threshold.precision.to_f64().unwrap(),
		})
		.collect::<Vec<_>>();
	let recall_series = props
		.precision_recall_curve_series
		.iter()
		.map(|threshold| LineChartPoint {
			x: threshold.threshold.to_f64().unwrap(),
			y: threshold.recall.to_f64().unwrap(),
		})
		.collect::<Vec<_>>();
	let parametric_series = vec![LineChartSeries {
		line_style: Some(LineStyle::Solid),
		point_style: Some(PointStyle::Circle),
		color: ui::colors::BLUE.to_owned(),
		data: pr_series,
		title: Some("PR".to_owned()),
	}];
	let non_parametric_series = vec![
		LineChartSeries {
			line_style: Some(LineStyle::Solid),
			point_style: Some(PointStyle::Circle),
			color: ui::colors::BLUE.to_owned(),
			data: precision_series,
			title: Some("Precision".to_owned()),
		},
		LineChartSeries {
			line_style: Some(LineStyle::Solid),
			point_style: Some(PointStyle::Circle),
			color: ui::colors::GREEN.to_owned(),
			data: recall_series,
			title: Some("Recall".to_owned()),
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
						selected={false}
						href="./"
					>
						{"Overview"}
					</ui::TabLink>
					<ui::TabLink
						disabled={false}
						href="precision_recall"
						selected={true}
						>
						{"PR Curve"}
					</ui::TabLink>
					<ui::TabLink
						href="roc"
						disabled={false}
						selected={false}
					>
						{"ROC Curve"}
					</ui::TabLink>
				</ui::TabBar>
				<ui::S2>
					<ui::H2 center={false}>{"Parametric Precision Recall Curve"}</ui::H2>
					<ui::P>{PRECISION_RECALL}</ui::P>
					<ui::Card>
						<LineChart
							class={None}
							labels={None}
							should_draw_x_axis_labels={None}
							should_draw_y_axis_labels={None}
							x_axis_grid_line_interval={None}
							y_axis_grid_line_interval={None}
							hide_legend={true}
							id={"parametric_pr".to_owned()}
							series={parametric_series}
							title={"Parametric Precision Recall Curve".to_owned()}
							x_axis_title={"Recall".to_owned()}
							x_max={1.0}
							x_min={0.0}
							y_axis_title={"Precision".to_owned()}
							y_max={1.0}
							y_min={0.0}
						/>
					</ui::Card>
				</ui::S2>
				<ui::S2>
					<ui::H2 center={false}>{"Non-Parametric Precision Recall Curve"}</ui::H2>
					<ui::P>{PRECISION_RECALL}</ui::P>
					<ui::Card>
						<LineChart
							class={None}
							labels={None}
							should_draw_x_axis_labels={None}
							should_draw_y_axis_labels={None}
							x_axis_grid_line_interval={None}
							y_axis_grid_line_interval={None}
							hide_legend={false}
							id={"non_parametric_pr".to_owned()}
							series={non_parametric_series}
							title={"Non-Parametric Precision Recall Curve".to_owned()}
							x_axis_title={"Threshold".to_owned()}
							x_max={1.0}
							x_min={0.0}
							y_axis_title={"Precision/Recall".to_owned()}
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
