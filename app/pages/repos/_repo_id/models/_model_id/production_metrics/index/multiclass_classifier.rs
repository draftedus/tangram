use crate::page::{
	AccuracyChart, ClassMetricsTableEntry, TrainingProductionMetrics, TrueValuesCountChartEntry,
};
use tangram_app_common::{
	date_window::{DateWindow, DateWindowInterval},
	date_window_select_field::DateWindowSelectField,
	definitions::{ACCURACY, PRECISION_RECALL},
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
pub struct MulticlassClassifierProductionMetricsProps {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub true_values_count_chart: Vec<TrueValuesCountChartEntry>,
	pub overall: MulticlassClassificationOverallProductionMetrics,
	pub id: String,
	pub accuracy_chart: AccuracyChart,
}

#[derive(Clone)]
pub struct MulticlassClassificationOverallProductionMetrics {
	pub accuracy: TrainingProductionMetrics,
	pub class_metrics_table: Vec<ClassMetricsTableEntry>,
	pub true_values_count: u64,
}

#[component]
pub fn MulticlassClassifierProductionMetrics(props: MulticlassClassifierProductionMetricsProps) {
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
			<ui::TabBar>
				<ui::TabLink
					href={"".to_owned()}
					selected={true}
					disabled={None}
				>
					{"Overview"}
				</ui::TabLink>
				<ui::TabLink
					href={format!("class_metrics?date_window={}", props.date_window)}
					selected={false}
					disabled={None}
				>
					{"Class Metrics"}
				</ui::TabLink>
			</ui::TabBar>
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
			<ClassMetricsTable
				class_metrics_table={props.overall.class_metrics_table}
			/>
		</ui::S1>
	}
}

#[component]
fn ClassMetricsTable(class_metrics_table: Vec<ClassMetricsTableEntry>) {
	html! {
	<ui::S2>
		<ui::H2 center={false}>{"Precision and Recall"}</ui::H2>
		<ui::P>{PRECISION_RECALL}</ui::P>
		<ui::Table width={"100%".to_owned()}>
			<ui::TableHeader>
				<ui::TableRow color={None}>
					<ui::TableHeaderCell
						color={None}
						expand={None}
						text_align={None}
					>
						{"Class"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						expand={None}
						text_align={None}
					>
						{"Training Precision"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						expand={None}
						text_align={None}
					>
						{"Training Recall"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						expand={None}
						text_align={None}
					>
						{"Production Precision"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						color={None}
						expand={None}
						text_align={None}
					>
						{"Production Recall"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
			{class_metrics_table.iter().map(|c| html! {
				<ui::TableRow color={None}>
					<ui::TableCell
						color={None}
						expand={None}
					>
						{c.class_name.to_owned()}
					</ui::TableCell>
					<ui::TableCell
						color={None}
						expand={None}
					>
						{ui::format_percent(c.precision.training)}
					</ui::TableCell>
					<ui::TableCell
						color={None}
						expand={None}
					>
						{ui::format_percent(c.recall.training)}
					</ui::TableCell>
					<ui::TableCell
						color={None}
						expand={None}
					>
						{c.precision.production.map(|precision| {
							ui::format_percent(precision)
						}).unwrap_or_else(|| "N/A".to_owned())}
					</ui::TableCell>
					<ui::TableCell
						color={None}
						expand={None}
					>
						{c.recall.production.map(|recall| {
							ui::format_percent(recall)
						}).unwrap_or_else(|| "N/A".to_owned())}
					</ui::TableCell>
				</ui::TableRow>
			}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	</ui::S2>
	}
}
