use crate::page::BinaryClassifierProps;
use html::{component, html};
use tangram_app_common::{
	definitions::ACCURACY,
	metrics_row::MetricsRow,
	tokens::{BASELINE_COLOR, TRAINING_COLOR},
};
use tangram_ui as ui;

#[component]
pub fn BinaryClassifierTrainingMetricsIndexPage(props: BinaryClassifierProps) {
	let value_formatter: fn(f32) -> String = |value| ui::format_percent(value);
	html! {
		<ui::S1>
		<ui::H1 center={false}>{"Training Metrics"}</ui::H1>
		<ui::TabBar>
			<ui::TabLink
				disabled={None}
				href={"./".to_owned()}
				selected={true}
			>
				{"Overview"}
			</ui::TabLink>
			<ui::TabLink
				disabled={None}
				selected={false}
				href={"precision_recall".to_owned()}
			>
				{"PR Curve"}
			</ui::TabLink>
			<ui::TabLink
				disabled={None}
				href={"roc".to_owned()}
				selected={false}
			>
				{"ROC Curve"}
			</ui::TabLink>
		</ui::TabBar>
		<ui::S2>
			<ui::P>
				{
					"At the end of training, your model was evaluated on a test dataset. All metrics in this section are from that evaluation. You can use these metrics to see how your model might perform on unseen production data."
				}
			</ui::P>
		</ui::S2>
		<ui::S2>
			<ui::H2 center={false}>{"Accuracy"}</ui::H2>
			<ui::P>{ACCURACY.to_string()}</ui::P>
			<ui::Card>
				<ui::NumberComparisonChart
					color_a={BASELINE_COLOR.to_string()}
					color_b={TRAINING_COLOR.to_string()}
					title={"Accuracy".to_owned()}
					value_a={props.baseline_accuracy}
					value_a_title={"Baseline".to_owned()}
					value_b={props.accuracy}
					value_b_title={"Training".to_owned()}
					value_formatter={value_formatter}
				/>
			</ui::Card>
		</ui::S2>
		<ui::Card>
			<ui::NumberChart
				title={"AUC ROC".to_owned()}
				value={ui::format_number(props.auc_roc)}
			/>
		</ui::Card>
		<MetricsRow>
			<ui::Card>
				<ui::NumberChart
					title={"Precision".to_owned()}
					value={ui::format_number(props.precision)}
				/>
			</ui::Card>
			<ui::Card>
				<ui::NumberChart
					title={"Recall".to_owned()}
					value={ui::format_number(props.recall)}
				/>
			</ui::Card>
			<ui::Card>
				<ui::NumberChart
					title={"F1 Score".to_owned()}
					value={ui::format_number(props.f1_score)}
				/>
			</ui::Card>
		</MetricsRow>
		</ui::S1>
	}
}
