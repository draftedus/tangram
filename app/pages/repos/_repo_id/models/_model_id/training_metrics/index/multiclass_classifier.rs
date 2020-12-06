use crate::page::MulticlassClassifierProps;
use tangram_app_common::{
	definitions::{ACCURACY, PRECISION_RECALL},
	tokens::{BASELINE_COLOR, TRAINING_COLOR},
};
use tangram_deps::html::{self, component, html};
use tangram_ui as ui;
use tangram_util::zip;

#[component]
pub fn MulticlassClassifierTrainingMetricsIndexPage(props: MulticlassClassifierProps) {
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
				href={"class_metrics".to_owned()}
				selected={false}
			>
				{"Class Metrics"}
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
					id={None}
					color_a={BASELINE_COLOR.to_string()}
					color_b={TRAINING_COLOR.to_string()}
					title={"Accuracy".to_owned()}
					value_a={props.baseline_accuracy}
					value_a_title={"Baseline Accuracy".to_owned()}
					value_b={props.accuracy}
					value_b_title={"Accuracy".to_owned()}
					value_formatter={value_formatter}
				/>
			</ui::Card>
		</ui::S2>
		<ui::S2>
			<ui::H2 center={false}>{"Precision and Recall"}</ui::H2>
			<ui::P>{PRECISION_RECALL}</ui::P>
			<ui::Table width={"100%".to_owned()}>
				<ui::TableHeader>
					<ui::TableRow color={None}>
						<ui::TableHeaderCell
							expand={false}
							text_align={None}
							color={None}
						>
							{"Class"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell
							expand={false}
							text_align={None}
							color={None}
						>
							{"Precision"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell
							expand={false}
							text_align={None}
							color={None}
						>
							{"Recall"}
						</ui::TableHeaderCell>
					</ui::TableRow>
				</ui::TableHeader>
				<ui::TableBody>
				{zip!(props.class_metrics, props.classes).map(|(class_metrics, class_name)| html! {
					<ui::TableRow color={None}>
						<ui::TableCell expand={None} color={None}>{class_name}</ui::TableCell>
						<ui::TableCell expand={None} color={None}>
							{ui::format_percent(class_metrics.precision)}
						</ui::TableCell>
						<ui::TableCell expand={None} color={None}>
							{ui::format_percent(class_metrics.recall)}
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
				</ui::TableBody>
			</ui::Table>
		</ui::S2>
	</ui::S1>
	}
}
