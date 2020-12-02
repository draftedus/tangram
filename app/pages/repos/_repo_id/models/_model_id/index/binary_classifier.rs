use super::page::TrainingSummary;
use html::html;
use num_traits::ToPrimitive;
use tangram_app_common::metrics_row::MetricsRow;
use tangram_charts::{
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_ui as ui;

pub struct BinaryClassifierProps {
	pub id: String,
	pub metrics: BinaryClassifierInnerMetrics,
	pub training_summary: TrainingSummary,
	pub losses_chart_series: Option<Vec<f32>>,
}

pub struct BinaryClassifierInnerMetrics {
	pub baseline_accuracy: f32,
	pub auc_roc: f32,
	pub accuracy: f32,
	pub precision: f32,
	pub recall: f32,
}

pub fn binary_classifier_index_page(props: BinaryClassifierProps) -> html::Node {
	let losses_chart_series = props.losses_chart_series.map(|losses_chart_series| {
		vec![LineChartSeries {
			line_style: Some(LineStyle::Solid),
			point_style: Some(PointStyle::Circle),
			color: ui::colors::BLUE.to_string(),
			data: losses_chart_series
				.iter()
				.enumerate()
				.map(|(index, loss)| LineChartPoint {
					x: index.to_f64().unwrap(),
					y: loss.to_f64().unwrap(),
				})
				.collect::<Vec<_>>(),
			title: Some("loss".to_owned()),
		}]
	});
	html! {
		<ui::S1>
			<ui::H1 center={false}>{"Overview"}</ui::H1>
			<ui::S2>
				<ui::H2 center={false}>{"Training Summary"}</ui::H2>
				<ui::P>
					{"Your dataset contained "}
					<b>
						{(props.training_summary.train_row_count +
							props.training_summary.test_row_count).to_string()}
					</b>
					{" rows and "}
					<b>{props.training_summary.column_count.to_string()}</b>
					{" columns. "}
					<b>{props.training_summary.train_row_count.to_string()}</b>
					{" of the rows were used in training and "}
					<b>{props.training_summary.test_row_count.to_string()}</b>
					{" were used in testing. The model with the highest "}
					<b>{props.training_summary.model_comparison_metric_type_name}</b>
					{" was chosen. The best model is a "}
					<b>{props.training_summary.chosen_model_type_name}</b>
					{"."}
				</ui::P>
			</ui::S2>
			<ui::S2>
				<ui::H2 center={false}>{"Metrics"}</ui::H2>
				<ui::P>
					{
						"Your model was evaluated on the test dataset and accurately classified "
					}
					<b>{ui::format_percent(props.metrics.accuracy)}</b>
					{" of the examples. This is compared with the baseline accuracy of "}
					<b>{ui::format_percent(props.metrics.baseline_accuracy)}</b>
					{
						", which is the accuracy achieved if the model always predicted the majority class."
					}
				</ui::P>
				<ui::Card>
					<ui::NumberChart
						title="AUC ROC"
						value={ui::format_number(props.metrics.auc_roc)}
					/>
				</ui::Card>
				<ui::Card>
					<ui::NumberChart
						title="Accuracy"
						value={ui::format_number(props.metrics.accuracy)}
					/>
				</ui::Card>
				<MetricsRow>
					<ui::Card>
						<ui::NumberChart
							title="Precision"
							value={ui::format_number(props.metrics.precision)}
						/>
					</ui::Card>
					<ui::Card>
						<ui::NumberChart
							title="Recall"
							value={ui::format_number(props.metrics.recall)}
						/>
					</ui::Card>
				</MetricsRow>
				{losses_chart_series.map(|losses_chart_series| html! {
					<LineChart
						labels={None}
						should_draw_x_axis_labels={None}
						should_draw_y_axis_labels={None}
						x_axis_grid_line_interval={None}
						x_max={None}
						x_min={None}
						y_axis_grid_line_interval={None}
						y_max={None}
						class={None}
						hide_legend={None}
						id={"loss_curve".to_owned()}
						series={losses_chart_series}
						title={"Training Loss Curve".to_owned()}
						x_axis_title={"Epoch".to_owned()}
						y_axis_title={"Loss".to_owned()}
						y_min={0.0}
					/>
				})}
			</ui::S2>
		</ui::S1>
	}
}
