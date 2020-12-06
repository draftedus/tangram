use crate::page::RegressorProps;
use tangram_app_common::{
	metrics_row::MetricsRow,
	tokens::{BASELINE_COLOR, TRAINING_COLOR},
};
use tangram_deps::html::{self, component, html};
use tangram_ui as ui;

#[component]
pub fn RegressorTrainingMetricsIndexPage(props: RegressorProps) {
	let value_formatter: fn(f32) -> String = |value| ui::format_number(value);
	html! {
		<ui::S1>
			<ui::H1 center={false}>{"Training Metrics"}</ui::H1>
			<ui::S2>
				<ui::P>
					{
						"At the end of training, your model was evaluated on a test dataset. All metrics in this section are from that evaluation. You can use these metrics to see how your model might perform on unseen production data."
					}
				</ui::P>
			</ui::S2>
			<ui::S2>
				<MetricsRow>
					<ui::Card>
						<ui::NumberComparisonChart
							id={None}
							color_a={BASELINE_COLOR.to_owned()}
							color_b={TRAINING_COLOR.to_owned()}
							title={"Root Mean Squared Error".to_owned()}
							value_a={props.baseline_rmse}
							value_a_title={"Baseline Mean Squared Error".to_owned()}
							value_b={props.rmse}
							value_b_title={"Root Mean Squared Error".to_owned()}
							value_formatter={value_formatter}
						/>
					</ui::Card>
					<ui::Card>
						<ui::NumberComparisonChart
							id={None}
							color_a={BASELINE_COLOR.to_owned()}
							color_b={TRAINING_COLOR.to_owned()}
							title={"Mean Squared Error".to_owned()}
							value_a={props.baseline_mse}
							value_a_title={"Mean Squared Error".to_owned()}
							value_b={props.mse}
							value_b_title={"Mean Squared Error".to_owned()}
							value_formatter={value_formatter}
						/>
					</ui::Card>
				</MetricsRow>
			</ui::S2>
		</ui::S1>
	}
}
