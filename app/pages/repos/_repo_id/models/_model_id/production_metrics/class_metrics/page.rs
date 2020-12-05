use tangram_app_common::{
	date_window::{DateWindow, DateWindowInterval},
	definitions::{CONFUSION_MATRIX, PRECISION_RECALL},
	metrics_row::MetricsRow,
	time::interval_chart_title,
	tokens::{PRODUCTION_COLOR, TRAINING_COLOR},
};
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_charts::{
	common::GridLineInterval,
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_deps::{
	html::{self, html},
	num_traits::ToPrimitive,
};
use tangram_ui as ui;

pub struct Props {
	pub id: String,
	pub class_metrics: Vec<ClassMetricsEntry>,
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub classes: Vec<String>,
	pub overall: OverallClassMetrics,
	pub model_layout_info: ModelLayoutInfo,
	pub class: String,
}

pub struct ClassMetricsEntry {
	pub class_name: String,
	pub intervals: Vec<IntervalEntry>,
}

pub struct IntervalEntry {
	pub label: String,
	pub f1_score: TrainingProductionMetrics,
	pub precision: TrainingProductionMetrics,
	pub recall: TrainingProductionMetrics,
}

pub struct OverallClassMetrics {
	pub class_metrics: Vec<OverallClassMetricsEntry>,
	pub label: String,
}

pub struct OverallClassMetricsEntry {
	pub class_name: String,
	pub comparison: Comparison,
	pub confusion_matrix: Option<ConfusionMatrix>,
	pub f1_score: TrainingProductionMetrics,
	pub precision: TrainingProductionMetrics,
	pub recall: TrainingProductionMetrics,
}

pub struct Comparison {
	pub false_negative_fraction: TrainingProductionMetrics,
	pub false_positive_fraction: TrainingProductionMetrics,
	pub true_positive_fraction: TrainingProductionMetrics,
	pub true_negative_fraction: TrainingProductionMetrics,
}

pub struct ConfusionMatrix {
	pub false_negatives: usize,
	pub true_negatives: usize,
	pub true_positives: usize,
	pub false_positives: usize,
}

pub struct TrainingProductionMetrics {
	pub production: Option<f32>,
	pub training: f32,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let selected_class_index = props
		.classes
		.iter()
		.position(|class| class == &props.class)
		.unwrap();
	let selected_class_interval_metrics = &props.class_metrics[selected_class_index];
	let selected_class_overall_metrics = &props.overall.class_metrics[selected_class_index];

	let precision_interval_chart_title =
		interval_chart_title(&props.date_window_interval, "Precision".to_owned());
	let recall_interval_chart_title =
		interval_chart_title(&props.date_window_interval, "Recall".to_owned());
	let f1_score_interval_chart_title =
		interval_chart_title(&props.date_window_interval, "F1 Score".to_owned());

	let chart_labels = selected_class_interval_metrics
		.intervals
		.iter()
		.map(|interval| interval.label.clone())
		.collect::<Vec<_>>();

	let precision_chart_series = vec![
		LineChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: selected_class_interval_metrics
				.intervals
				.iter()
				.enumerate()
				.map(|(index, interval)| LineChartPoint {
					x: index.to_f64().unwrap(),
					y: interval.precision.training.to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
			line_style: Some(LineStyle::Dashed),
			point_style: Some(PointStyle::Hidden),
			title: Some("Training Precision".to_owned()),
		},
		LineChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: selected_class_interval_metrics
				.intervals
				.iter()
				.enumerate()
				.map(|(index, interval)| LineChartPoint {
					x: index.to_f64().unwrap(),
					y: interval.precision.production.unwrap().to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
			line_style: None,
			point_style: None,
			title: Some("Production Precision".to_owned()),
		},
	];
	let recall_chart_series = vec![
		LineChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: selected_class_interval_metrics
				.intervals
				.iter()
				.enumerate()
				.map(|(index, interval)| LineChartPoint {
					x: index.to_f64().unwrap(),
					y: interval.recall.training.to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
			line_style: Some(LineStyle::Dashed),
			point_style: Some(PointStyle::Hidden),
			title: Some("Training Recall".to_owned()),
		},
		LineChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: selected_class_interval_metrics
				.intervals
				.iter()
				.enumerate()
				.map(|(index, interval)| LineChartPoint {
					x: index.to_f64().unwrap(),
					y: interval.recall.production.unwrap().to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
			line_style: None,
			point_style: None,
			title: Some("Production Recall".to_owned()),
		},
	];
	let f1_score_chart_series = vec![
		LineChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: selected_class_interval_metrics
				.intervals
				.iter()
				.enumerate()
				.map(|(index, interval)| LineChartPoint {
					x: index.to_f64().unwrap(),
					y: interval.f1_score.training.to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
			line_style: Some(LineStyle::Dashed),
			point_style: Some(PointStyle::Hidden),
			title: Some("Training F1 Score".to_owned()),
		},
		LineChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: selected_class_interval_metrics
				.intervals
				.iter()
				.enumerate()
				.map(|(index, interval)| LineChartPoint {
					x: index.to_f64().unwrap(),
					y: interval.f1_score.production.unwrap().to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
			line_style: None,
			point_style: None,
			title: Some("Production F1 Score".to_owned()),
		},
	];
	let value_formatter: fn(f32) -> String = |value: f32| ui::format_percent(value);

	let html = html! {
	<ModelLayout
		info={props.model_layout_info}
		page_info={page_info}
		selected_item={ModelSideNavItem::ProductionMetrics}
	>
		<ui::S1>
			<ui::H1 center={false}>{"Production Metrics"}</ui::H1>
			<ui::TabBar>
				<ui::TabLink
					href={"./".to_owned()}
					selected={false}
					disabled={None}
				>
					{"Overview"}
				</ui::TabLink>
				<ui::TabLink
					href={"class_metrics".to_owned()}
					selected={false}
					disabled={None}
				>
					{"Class Metrics"}
				</ui::TabLink>
			</ui::TabBar>
			<ui::Form
				action={None}
				autocomplete={None}
				enc_type={None}
				id={None}
				post={None}
			>
		// 		<DateWindowSelectField dateWindow={props.dateWindow} />
		// 		<ClassSelectField class={props.class} classes={props.classes} />
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
			<ui::S2>
				<ui::H2 center={false}>{"Precision and Recall"}</ui::H2>
					<ui::P>{PRECISION_RECALL}</ui::P>
					<MetricsRow>
						<ui::Card>
							<ui::NumberComparisonChart
								color_a={TRAINING_COLOR.to_owned()}
								color_b={PRODUCTION_COLOR.to_owned()}
								title={"Precision".to_owned()}
								value_a={selected_class_overall_metrics.precision.training}
								value_a_title={"Training".to_owned()}
								value_b={selected_class_overall_metrics.precision.production.unwrap()}
								value_b_title={"Production".to_owned()}
								value_formatter={value_formatter}
							/>
							</ui::Card>
							<ui::Card>
								<ui::NumberComparisonChart
									color_a={TRAINING_COLOR.to_owned()}
									color_b={PRODUCTION_COLOR.to_owned()}
									title={"Recall".to_owned()}
									value_a={selected_class_overall_metrics.recall.training}
									value_a_title={"Training".to_owned()}
									value_b={selected_class_overall_metrics.recall.production.unwrap()}
									value_b_title={"Production".to_owned()}
									value_formatter={value_formatter}
								/>
							</ui::Card>
						</MetricsRow>
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
								id={"precision_intervals".to_owned()}
								labels={chart_labels.clone()}
								series={precision_chart_series}
								title={precision_interval_chart_title}
								x_axis_grid_line_interval={Some(GridLineInterval { k: 1.0, p: 0.0 })}
								y_axis_grid_line_interval={None}
								y_max={1.0}
								y_min={0.0}
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
								x_axis_grid_line_interval={
									Some(GridLineInterval { k: 1.0, p: 0.0 })
								}
								y_axis_grid_line_interval={None}
								y_max={1.0}
								y_min={0.0}
								id={"recall_intervals".to_owned()}
								labels={chart_labels.clone()}
								series={recall_chart_series}
								title={recall_interval_chart_title}
							/>
						</ui::Card>
						<MetricsRow>
							<ui::Card>
								<ui::NumberComparisonChart
									color_a={TRAINING_COLOR.to_owned()}
									color_b={PRODUCTION_COLOR.to_owned()}
									title={"F1 Score".to_owned()}
									value_a={selected_class_overall_metrics.f1_score.training}
									value_a_title={"Training".to_owned()}
									value_b={selected_class_overall_metrics.f1_score.production.unwrap()}
									value_b_title={"Production".to_owned()}
									value_formatter={value_formatter}
								/>
							</ui::Card>
						</MetricsRow>
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
								x_axis_grid_line_interval={
									Some(GridLineInterval { k: 1.0, p: 0.0 })
								}
								y_axis_grid_line_interval={None}
								y_max={1.0}
								y_min={0.0}
								id={"f1_intervals".to_owned()}
								labels={chart_labels}
								series={f1_score_chart_series}
								title={f1_score_interval_chart_title}
							/>
						</ui::Card>
					</ui::S2>
					<ui::S2>
						<ui::H2 center={false}>{"Confusion Matrix"}</ui::H2>
						<ui::P>{CONFUSION_MATRIX}</ui::P>
						<ui::ConfusionMatrix
							class_label={props.class.to_owned()}
							false_negatives={
								selected_class_overall_metrics.confusion_matrix.as_ref().unwrap().false_negatives
							}
							false_positives={
								selected_class_overall_metrics.confusion_matrix.as_ref().unwrap().false_positives
							}
							true_negatives={
								selected_class_overall_metrics.confusion_matrix.as_ref().unwrap().true_negatives
							}
							true_positives={
								selected_class_overall_metrics.confusion_matrix.as_ref().unwrap().true_positives
							}
						/>
					</ui::S2>
					<ui::S2>
						<ui::H2 center={false}>{"Production v. Training Confusion Matrix"}</ui::H2>
						<ui::P>{CONFUSION_MATRIX}</ui::P>
						<ui::ConfusionMatrixComparison
							class_label={props.class.to_owned()}
							color_a={TRAINING_COLOR.to_owned()}
							color_b={PRODUCTION_COLOR.to_owned()}
							value_a={ui::ConfusionMatrixComparisonValue {
								false_negative:
									selected_class_overall_metrics.comparison
										.false_negative_fraction.training,
								false_positive:
									selected_class_overall_metrics.comparison
										.false_positive_fraction.training,
								true_negative:
									selected_class_overall_metrics.comparison
										.true_negative_fraction.training,
								true_positive:
									selected_class_overall_metrics.comparison
										.true_positive_fraction.training,
							}}
							value_a_title={"Training".to_owned()}
							value_b={ui::ConfusionMatrixComparisonValue {
								false_negative:
									selected_class_overall_metrics.comparison
										.false_negative_fraction.production.unwrap(),
								false_positive:
									selected_class_overall_metrics.comparison
										.false_positive_fraction.production.unwrap(),
								true_negative:
									selected_class_overall_metrics.comparison
										.true_negative_fraction.production.unwrap(),
								true_positive:
									selected_class_overall_metrics.comparison
										.true_positive_fraction.production.unwrap(),
							}}
							value_b_title={"Production".to_owned()}
							value_formatter={value_formatter}
						/>
				</ui::S2>
			</ui::S1>
		</ModelLayout>
	};
	html.render_to_string()
}
