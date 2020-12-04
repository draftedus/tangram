use crate::page::{IntervalBoxChartDataPoint, OverallBoxChartData};
use tangram_app_common::{
	date_window::{DateWindow, DateWindowInterval},
	metrics_row::MetricsRow,
	time::{interval_chart_title, overall_chart_title},
	tokens::{PRODUCTION_COLOR, TRAINING_COLOR},
};
use tangram_charts::box_chart::{BoxChartPoint, BoxChartSeries, BoxChartValue};
use tangram_charts::components::BoxChart;
use tangram_deps::{
	html::{self, component, html},
	num_traits::ToPrimitive,
};
use tangram_ui as ui;

#[derive(Clone)]
pub struct NumberColumnProps {
	pub absent_count: u64,
	pub alert: Option<String>,
	pub column_name: String,
	pub date_window_interval: DateWindowInterval,
	pub date_window: DateWindow,
	pub interval_box_chart_data: Vec<IntervalBoxChartDataPoint>,
	pub invalid_count: u64,
	pub max_comparison: NumberTrainingProductionComparison,
	pub mean_comparison: NumberTrainingProductionComparison,
	pub min_comparison: NumberTrainingProductionComparison,
	pub overall_box_chart_data: OverallBoxChartData,
	pub row_count: u64,
	pub std_comparison: NumberTrainingProductionComparison,
}

#[derive(Clone)]
pub struct NumberTrainingProductionComparison {
	pub production: Option<f32>,
	pub training: f32,
}

#[component]
pub fn NumberColumn(props: NumberColumnProps) {
	let interval_box_chart_series = vec![BoxChartSeries {
		color: PRODUCTION_COLOR.to_owned(),
		data: props
			.interval_box_chart_data
			.iter()
			.enumerate()
			.map(|(index, entry)| BoxChartPoint {
				label: entry.label.to_owned(),
				x: index.to_f64().unwrap(),
				y: entry.stats.as_ref().map(|stats| BoxChartValue {
					max: stats.max.to_f64().unwrap(),
					min: stats.min.to_f64().unwrap(),
					p25: stats.p25.to_f64().unwrap(),
					p50: stats.p50.to_f64().unwrap(),
					p75: stats.p75.to_f64().unwrap(),
				}),
			})
			.collect(),
		title: Some(format!("Production Stats for {}", props.column_name)),
	}];
	let overall_box_chart_series = vec![
		BoxChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: vec![BoxChartPoint {
				label: "Training".to_owned(),
				x: 0.0,
				y: Some(BoxChartValue {
					max: props.overall_box_chart_data.training.max.to_f64().unwrap(),
					min: props.overall_box_chart_data.training.min.to_f64().unwrap(),
					p25: props.overall_box_chart_data.training.p25.to_f64().unwrap(),
					p50: props.overall_box_chart_data.training.p50.to_f64().unwrap(),
					p75: props.overall_box_chart_data.training.p75.to_f64().unwrap(),
				}),
			}],
			title: Some(format!("Training Stats for {}", props.column_name)),
		},
		BoxChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: vec![BoxChartPoint {
				label: "Training".to_owned(),
				x: 0.0,
				y: props
					.overall_box_chart_data
					.production
					.map(|production| BoxChartValue {
						max: production.max.to_f64().unwrap(),
						min: production.min.to_f64().unwrap(),
						p25: production.p25.to_f64().unwrap(),
						p50: production.p50.to_f64().unwrap(),
						p75: production.p75.to_f64().unwrap(),
					}),
			}],
			title: Some(format!("Production Stats for {}", props.column_name)),
		},
	];
	let stats_overall_chart_title = overall_chart_title(props.date_window, "Stats".to_owned());
	let stats_interval_chart_title =
		interval_chart_title(props.date_window_interval, "Stats".to_owned());
	let value_formatter: fn(f32) -> String = |value: f32| ui::format_number(value);
	html! {
		<ui::S2>
			{props.alert.map(|alert| html! {
				<ui::Alert
					title={None}
					level={ui::Level::Danger}
				>
					{alert}
				</ui::Alert>
			})}
			<ui::Card>
				<BoxChart
					class={None}
					hide_legend={None}
					should_draw_x_axis_labels={None}
					should_draw_y_axis_labels={None}
					x_axis_title={None}
					y_axis_title={None}
					y_max={None}
					y_min={None}
					id={"number_overall".to_owned()}
					series={overall_box_chart_series}
					title={stats_overall_chart_title}
				/>
			</ui::Card>
			<ui::Card>
				<BoxChart
					class={None}
					hide_legend={None}
					should_draw_x_axis_labels={None}
					should_draw_y_axis_labels={None}
					x_axis_title={None}
					y_axis_title={None}
					y_max={None}
					y_min={None}
					id={"number_intervals".to_owned()}
					series={interval_box_chart_series}
					title={stats_interval_chart_title}
				/>
			</ui::Card>
			<MetricsRow>
				<ui::Card>
					<ui::NumberChart
						title={"Row Count".to_owned()}
						value={props.row_count.to_string()}
					/>
				</ui::Card>
				<ui::Card>
					<ui::NumberChart
						title={"Absent Count".to_owned()}
						value={props.absent_count.to_string()}
					/>
				</ui::Card>
				<ui::Card>
					<ui::NumberChart
						title={"Invalid Count".to_owned()}
						value={props.invalid_count.to_string()}
					/>
				</ui::Card>
			</MetricsRow>
			<MetricsRow>
				<ui::Card>
					<ui::NumberComparisonChart
						color_a={TRAINING_COLOR.to_owned()}
						color_b={PRODUCTION_COLOR.to_owned()}
						title={"Min".to_owned()}
						value_a={props.min_comparison.training}
						value_a_title={"Training".to_owned()}
						value_b={props.min_comparison.production.unwrap()}
						value_b_title={"Production".to_owned()}
						value_formatter={value_formatter}
					/>
				</ui::Card>
				<ui::Card>
					<ui::NumberComparisonChart
						color_a={TRAINING_COLOR.to_owned()}
						color_b={PRODUCTION_COLOR.to_owned()}
						title={"Max".to_owned()}
						value_a={props.max_comparison.training}
						value_a_title={"Training".to_owned()}
						value_b={props.max_comparison.production.unwrap()}
						value_b_title={"Production".to_owned()}
						value_formatter={value_formatter}
					/>
				</ui::Card>
			</MetricsRow>
			<MetricsRow>
				<ui::Card>
					<ui::NumberComparisonChart
						color_a={TRAINING_COLOR.to_owned()}
						color_b={PRODUCTION_COLOR.to_owned()}
						title={"Mean".to_owned()}
						value_a={props.mean_comparison.training}
						value_a_title={"Training".to_owned()}
						value_b={props.mean_comparison.production.unwrap()}
						value_b_title={"Production".to_owned()}
						value_formatter={value_formatter}
					/>
				</ui::Card>
				<ui::Card>
					<ui::NumberComparisonChart
						color_a={TRAINING_COLOR.to_owned()}
						color_b={PRODUCTION_COLOR.to_owned()}
						title={"Standard Deviation".to_owned()}
						value_a={props.std_comparison.training}
						value_a_title={"Training".to_owned()}
						value_b={props.std_comparison.production.unwrap()}
						value_b_title={"Production".to_owned()}
						value_formatter={value_formatter}
					/>
				</ui::Card>
			</MetricsRow>
		</ui::S2>
	}
}
