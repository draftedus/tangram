use html::html;
use num_traits::ToPrimitive;
use tangram_app_common::{
	definitions::{CONFUSION_MATRIX, PRECISION_RECALL},
	metrics_row::MetricsRow,
};
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_ui as ui;

pub struct Props {
	pub class: String,
	pub classes: Vec<String>,
	pub f1_score: f32,
	pub false_negatives: u64,
	pub false_positives: u64,
	pub id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub precision: f32,
	pub recall: f32,
	pub true_negatives: u64,
	pub true_positives: u64,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let html = html! {
		<ModelLayout
			info={props.model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::TrainingMetrics}
		>
			<ui::S1>
				<ui::H1 center={false}>{"Training Metrics"}</ui::H1>
				<ui::TabBar>
					<ui::TabLink
						disabled={None}
						selected={false}
						href={"./".to_owned()}
					>
						{"Overview"}
					</ui::TabLink>
					<ui::TabLink
						disabled={None}
						selected={true}
						href={"class_metrics".to_owned()}
					>
					{"Class Metrics"}
					</ui::TabLink>
				</ui::TabBar>
				<ui::Form
					enc_type={None}
					post={None}
					id={None}
					action={None}
					autocomplete={None}
				>
				// 	<ClassSelectField class={props.class} classes={props.classes} />
					<noscript>
						<ui::Button
							color={None}
							disabled={None}
							button_type={ui::ButtonType::Button}
							download={None}
							href={None}
							id={None}
						>
							{"Submit"}
						</ui::Button>
					</noscript>
				</ui::Form>
				<ui::S2>
					<ui::H2 center={false}>{"Precision and Recall"}</ui::H2>
					<ui::P>{PRECISION_RECALL.to_string()}</ui::P>
					<MetricsRow>
						<ui::Card>
							<ui::NumberChart
								title="Precision"
								value={ui::format_percent(props.precision)}
							/>
						</ui::Card>
						<ui::Card>
							<ui::NumberChart
								title="Recall"
								value={ui::format_percent(props.recall)}
							/>
						</ui::Card>
					</MetricsRow>
					<MetricsRow>
						<ui::Card>
							<ui::NumberChart
								title={"F1 Score".to_owned()}
								value={ui::format_percent(props.f1_score)}
							/>
						</ui::Card>
					</MetricsRow>
				</ui::S2>
				<ui::S2>
					<ui::H2 center={false}>{"Confusion Matrix"}</ui::H2>
					<ui::P>{CONFUSION_MATRIX}</ui::P>
					<ui::ConfusionMatrix
						class_label={props.class}
						false_negatives={props.false_negatives.to_usize().unwrap()}
						false_positives={props.false_positives.to_usize().unwrap()}
						true_negatives={props.true_negatives.to_usize().unwrap()}
						true_positives={props.true_positives.to_usize().unwrap()}
					/>
				</ui::S2>
			</ui::S1>
		</ModelLayout>
	};
	html.render_to_string()
}
