use tangram_app_common::{
	metrics_row::MetricsRow,
	tokens::{BASELINE_COLOR, SELECTED_THRESHOLD_COLOR},
};
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_deps::{
	html::{self, component, html},
	num_traits::ToPrimitive,
};
use tangram_ui as ui;

pub struct Props {
	pub tuning: Option<TuningProps>,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(Clone)]
pub struct TuningProps {
	pub baseline_threshold: f32,
	pub metrics: Vec<Metrics>,
	pub class: String,
}

#[derive(Clone)]
pub struct Metrics {
	pub accuracy: f32,
	pub f1_score: f32,
	pub false_negatives: u64,
	pub false_positives: u64,
	pub precision: f32,
	pub recall: f32,
	pub threshold: f32,
	pub true_negatives: u64,
	pub true_positives: u64,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let inner = match props.tuning {
		Some(tuning_props) => html! {<Tuning props={tuning_props}/> },
		None => html! {
			<ui::S1>
				<ui::P>{"Tuning is not supported for this model."}</ui::P>
			</ui::S1>
		},
	};
	let html = html! {
		<ModelLayout
			info={props.model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::Tuning}
		>
			{inner}
		</ModelLayout>
	};
	html.render_to_string()
}

#[component]
fn Tuning(props: TuningProps) {
	let thresholds = props
		.metrics
		.iter()
		.map(|metric| metric.threshold)
		.collect::<Vec<_>>();
	let baseline_index = thresholds
		.iter()
		.position(|value| (value - props.baseline_threshold).abs() < std::f32::EPSILON)
		.unwrap();
	let selected_threshold_index = baseline_index;
	let selected_threshold = thresholds[selected_threshold_index];
	let baseline_metrics = &props.metrics[baseline_index];
	let selected_threshold_metrics = &props.metrics[selected_threshold_index];
	let value_formatter: fn(f32) -> String = ui::format_percent;
	html! {
		<ui::S1>
			<ui::H1 center={false}>{"Tuning"}</ui::H1>
			<ui::S2>
				<ui::P>
					{
						"Drag the silder to see how metrics change with varying settings of the threshold."
					}
				</ui::P>
				<ui::Slider
					id={"slider".to_owned()}
					max={(thresholds.len() - 1).to_f32().unwrap()}
					min={0.0}
					value={selected_threshold_index}
				/>
			</ui::S2>
			{if selected_threshold == 0.0 {
				Some(html! {
					<ui::Alert
						title={None}
						level={ui::Level::Info}
					>
						{"A threshold of 0 makes your model predict the same class for every input."}
					</ui::Alert>
				})
				} else if selected_threshold.partial_cmp(&1.0).unwrap() == std::cmp::Ordering::Equal {
					Some(html! {
						<ui::Alert
							title={None}
							level={ui::Level::Info}
						>
							{"A threshold of 1 makes your model predict the same class for every input."}
						</ui::Alert>
					})
				} else {
					None
			}}
			<ui::S2>
				<MetricsRow>
					<ui::Card>
						<ui::NumberComparisonChart
							color_a={BASELINE_COLOR.to_owned()}
							color_b={SELECTED_THRESHOLD_COLOR.to_owned()}
							title={"Accuracy".to_owned()}
							value_a={baseline_metrics.accuracy}
							value_a_title={"Baseline".to_owned()}
							value_b={selected_threshold_metrics.accuracy}
							value_b_title={"Selected Threshold".to_owned()}
							value_formatter={value_formatter}
						/>
					</ui::Card>
					<ui::Card>
						<ui::NumberComparisonChart
							color_a={BASELINE_COLOR.to_owned()}
							color_b={SELECTED_THRESHOLD_COLOR.to_owned()}
							title={"F1 Score".to_owned()}
							value_a={baseline_metrics.f1_score}
							value_a_title={"Baseline".to_owned()}
							value_b={selected_threshold_metrics.f1_score}
							value_b_title={"Selected Threshold".to_owned()}
							value_formatter={value_formatter}
						/>
					</ui::Card>
				</MetricsRow>
				<MetricsRow>
					<ui::Card>
						<ui::NumberComparisonChart
							color_a={BASELINE_COLOR.to_owned()}
							color_b={SELECTED_THRESHOLD_COLOR.to_owned()}
							title={"Precision".to_owned()}
							value_a={baseline_metrics.precision}
							value_a_title={"Baseline".to_owned()}
							value_b={selected_threshold_metrics.precision}
							value_b_title={"Selected Threshold".to_owned()}
							value_formatter={value_formatter}
						/>
					</ui::Card>
					<ui::Card>
						<ui::NumberComparisonChart
							color_a={BASELINE_COLOR.to_owned()}
							color_b={SELECTED_THRESHOLD_COLOR.to_owned()}
							title={"Recall".to_owned()}
							value_a={baseline_metrics.recall}
							value_a_title={"Baseline".to_owned()}
							value_b={selected_threshold_metrics.recall}
							value_b_title="Selected Threshold"
							value_formatter={value_formatter}
						/>
					</ui::Card>
				</MetricsRow>
			</ui::S2>
			<ui::S2>
				<ui::ConfusionMatrixComparison
					class_label={props.class.to_owned()}
					color_a={BASELINE_COLOR.to_owned()}
					color_b={SELECTED_THRESHOLD_COLOR.to_owned()}
					value_a={ui::ConfusionMatrixComparisonValue {
						false_negative: baseline_metrics.false_negatives.to_f32().unwrap(),
						false_positive: baseline_metrics.false_positives.to_f32().unwrap(),
						true_negative: baseline_metrics.true_negatives.to_f32().unwrap(),
						true_positive: baseline_metrics.true_positives.to_f32().unwrap(),
					}}
					value_a_title="Baseline"
					value_b={ui::ConfusionMatrixComparisonValue {
						false_negative: selected_threshold_metrics.false_negatives.to_f32().unwrap(),
						false_positive: selected_threshold_metrics.false_positives.to_f32().unwrap(),
						true_negative: selected_threshold_metrics.true_negatives.to_f32().unwrap(),
						true_positive: selected_threshold_metrics.true_positives.to_f32().unwrap(),
					}}
					value_b_title={"Selected Threshold".to_owned()}
					value_formatter={None}
				/>
			</ui::S2>
		</ui::S1>
	}
}
