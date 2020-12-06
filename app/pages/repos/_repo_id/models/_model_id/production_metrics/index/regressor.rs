use crate::page::{TrainingProductionMetrics, TrueValuesCountChartEntry};
use tangram_app_common::{
	date_window::{DateWindow, DateWindowInterval},
	date_window_select_field::DateWindowSelectField,
	metrics_row::MetricsRow,
	time::interval_chart_title,
	tokens::{PRODUCTION_COLOR, TRAINING_COLOR},
};
use tangram_charts::{
	common::GridLineInterval,
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_deps::{
	html::{self, component, html},
	num_traits::ToPrimitive,
};
use tangram_ui as ui;

#[derive(Clone)]
pub struct RegressorProductionMetricsProps {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub mse_chart: MSEChart,
	pub overall: RegressionProductionMetrics,
	pub true_values_count_chart: Vec<TrueValuesCountChartEntry>,
}

#[derive(Clone)]
pub struct MSEChart {
	pub data: Vec<MSEChartEntry>,
	pub training_mse: f32,
}

#[derive(Clone)]
pub struct MSEChartEntry {
	pub label: String,
	pub mse: Option<f32>,
}

#[derive(Clone)]
pub struct RegressionProductionMetrics {
	pub mse: TrainingProductionMetrics,
	pub rmse: TrainingProductionMetrics,
	pub true_values_count: u64,
}

#[component]
pub fn RegressorProductionMetrics(props: RegressorProductionMetricsProps) {
	let mse_chart_labels = props
		.mse_chart
		.data
		.iter()
		.map(|entry| entry.label.clone())
		.collect::<Vec<_>>();
	let mse_series = vec![
		LineChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: (0..props.mse_chart.data.len())
				.map(|index| LineChartPoint {
					x: index.to_f64().unwrap(),
					y: props.mse_chart.training_mse.to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
			line_style: Some(LineStyle::Dashed),
			point_style: Some(PointStyle::Hidden),
			title: Some("Training Mean Squared Error".to_owned()),
		},
		LineChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: props
				.mse_chart
				.data
				.iter()
				.enumerate()
				.map(|(index, entry)| LineChartPoint {
					x: index.to_f64().unwrap(),
					y: entry.mse.unwrap().to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
			line_style: None,
			point_style: None,
			title: Some("Production Mean Squared Error".to_owned()),
		},
	];
	let mse_chart_title =
		interval_chart_title(&props.date_window_interval, "Mean Squared Error".to_owned());
	let value_formatter: fn(f32) -> String = |value: f32| ui::format_number(value);
	html! {
	<ui::S1>
		<ui::H1 center={false}>{"Production Metrics"}</ui::H1>
		<ui::S2>
			<ui::Form
				action={None}
				autocomplete={None}
				enc_type={None}
				id={None}
				post={None}
			>
				<DateWindowSelectField date_window={props.date_window} />
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
			<ui::P>
				{"You have logged "}
				<b>{props.overall.true_values_count.to_string()}</b>
				{" true values for this date range."}
			</ui::P>
			<ui::Card>
				<LineChart
					hide_legend={None}
					x_axis_title={None}
					x_min={None}
					x_max={None}
					class={None}
					should_draw_x_axis_labels={None}
					should_draw_y_axis_labels={None}
					id={"mse".to_owned()}
					labels={mse_chart_labels}
					series={mse_series}
					title={mse_chart_title}
					x_axis_grid_line_interval={
						Some(GridLineInterval { k: 1.0, p: 0.0 })
					}
					y_axis_title={"Root Mean Squared Error".to_string()}
					y_axis_grid_line_interval={None}
					y_max={1.0}
					y_min={0.0}
				/>
			</ui::Card>
			<MetricsRow>
				<ui::Card>
					<ui::NumberChart
						title="True Value Count"
						value={props.overall.true_values_count.to_string()}
					/>
				</ui::Card>
			</MetricsRow>
			<MetricsRow>
				<ui::Card>
					<ui::NumberComparisonChart
						id={None}
						color_a={TRAINING_COLOR.to_owned()}
						color_b={PRODUCTION_COLOR.to_owned()}
						title={"Root Mean Squared Error".to_owned()}
						value_a={props.overall.rmse.training}
						value_a_title={"Training".to_owned()}
						value_b={props.overall.rmse.production.unwrap()}
						value_b_title={"Production".to_owned()}
						value_formatter={value_formatter}
					/>
				</ui::Card>
				<ui::Card>
					<ui::NumberComparisonChart
						id={None}
						color_a={TRAINING_COLOR.to_owned()}
						color_b={PRODUCTION_COLOR.to_owned()}
						title={"Mean Squared Error".to_owned()}
						value_a={props.overall.mse.training}
						value_a_title={"Training".to_owned()}
						value_b={props.overall.mse.production.unwrap()}
						value_b_title={"Production".to_owned()}
						value_formatter={value_formatter}
					/>
				</ui::Card>
			</MetricsRow>
		</ui::S2>
	</ui::S1>
	}
}
