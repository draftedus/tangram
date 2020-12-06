use crate::page::{AccuracyChart, TrainingProductionMetrics, TrueValuesCountChartEntry};
use tangram_app_common::{
	date_window::{DateWindow, DateWindowInterval},
	date_window_select_field::DateWindowSelectField,
	definitions::ACCURACY,
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
pub struct BinaryClassifierProductionMetricsProps {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub true_values_count_chart: Vec<TrueValuesCountChartEntry>,
	pub overall: BinaryClassificationOverallProductionMetrics,
	pub id: String,
	pub accuracy_chart: AccuracyChart,
}

#[derive(Clone)]
pub struct BinaryClassificationOverallProductionMetrics {
	pub accuracy: TrainingProductionMetrics,
	pub precision: TrainingProductionMetrics,
	pub recall: TrainingProductionMetrics,
	pub true_values_count: u64,
}

#[component]
pub fn BinaryClassifierProductionMetrics(props: BinaryClassifierProductionMetricsProps) {
	let chart_labels = props
		.accuracy_chart
		.data
		.iter()
		.map(|entry| entry.label.clone())
		.collect::<Vec<_>>();
	let accuracy_series = vec![
		LineChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: (0..props.accuracy_chart.data.len())
				.map(|index| LineChartPoint {
					x: index.to_f64().unwrap(),
					y: props.accuracy_chart.training_accuracy.to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
			line_style: Some(LineStyle::Dashed),
			point_style: Some(PointStyle::Hidden),
			title: Some("Training Accuracy".to_owned()),
		},
		LineChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: props
				.accuracy_chart
				.data
				.iter()
				.enumerate()
				.map(|(index, entry)| LineChartPoint {
					x: index.to_f64().unwrap(),
					y: entry.accuracy.unwrap().to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
			line_style: None,
			point_style: None,
			title: Some("Production Accuracy".to_owned()),
		},
	];
	let accuracy_chart_title =
		interval_chart_title(&props.date_window_interval, "Accuracy".to_owned());
	let value_formatter: fn(f32) -> String = |value: f32| ui::format_percent(value);
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
				<MetricsRow>
					<ui::Card>
						<ui::NumberChart
							title={"True Value Count".to_owned()}
							value={props.overall.true_values_count.to_string()}
						/>
					</ui::Card>
				</MetricsRow>
			</ui::S2>
			<ui::S2>
				<ui::H2 center={false}>{"Accuracy"}</ui::H2>
				<ui::P>{ACCURACY}</ui::P>
				<ui::Card>
					<ui::NumberComparisonChart
						id={None}
						color_a={TRAINING_COLOR.to_owned()}
						color_b={PRODUCTION_COLOR.to_owned()}
						title={"Accuracy".to_owned()}
						value_a={props.overall.accuracy.training}
						value_a_title={"Training".to_owned()}
						value_b={props.overall.accuracy.production.unwrap()}
						value_b_title={"Production".to_owned()}
						value_formatter={value_formatter}
					/>
				</ui::Card>
				<ui::Card>
					<LineChart
						hide_legend={None}
						x_axis_title={None}
						x_min={None}
						x_max={None}
						y_axis_title={None}
						class={None}
						should_draw_x_axis_labels={None}
						should_draw_y_axis_labels={None}
						id={"accuracy".to_owned()}
						labels={chart_labels}
						series={accuracy_series}
						title={accuracy_chart_title}
						x_axis_grid_line_interval={
							Some(GridLineInterval { k: 1.0, p: 0.0 })
						}
						y_axis_grid_line_interval={None}
						y_max={1.0}
						y_min={0.0}
					/>
				</ui::Card>
			</ui::S2>
		</ui::S1>
	}
}
