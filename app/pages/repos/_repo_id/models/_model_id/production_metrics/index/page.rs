use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_deps::html::{self, html};

pub use crate::binary_classifier::*;
pub use crate::multiclass_classifier::*;
pub use crate::regressor::*;

#[derive(Clone)]
pub struct Props {
	pub id: String,
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(Clone)]
pub enum Inner {
	Regressor(RegressorProductionMetricsProps),
	BinaryClassifier(BinaryClassifierProductionMetricsProps),
	MulticlassClassifier(MulticlassClassifierProductionMetricsProps),
}

#[derive(Clone)]
pub struct TrueValuesCountChartEntry {
	pub label: String,
	pub count: u64,
}

#[derive(Clone)]
pub struct TrainingProductionMetrics {
	pub production: Option<f32>,
	pub training: f32,
}

#[derive(Clone)]
pub struct AccuracyChart {
	pub data: Vec<AccuracyChartEntry>,
	pub training_accuracy: f32,
}

#[derive(Clone)]
pub struct AccuracyChartEntry {
	pub accuracy: Option<f32>,
	pub label: String,
}

#[derive(Clone)]
pub struct ClassMetricsTableEntry {
	pub class_name: String,
	pub precision: TrainingProductionMetrics,
	pub recall: TrainingProductionMetrics,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let inner = match props.inner {
		Inner::Regressor(inner) => html! {
			<RegressorProductionMetrics props={inner} />
		},
		Inner::BinaryClassifier(inner) => html! {
			<BinaryClassifierProductionMetrics props={inner} />
		},
		Inner::MulticlassClassifier(inner) => html! {
			<MulticlassClassifierProductionMetrics props={inner} />
		},
	};
	let html = html! {
		<ModelLayout
			info={props.model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::ProductionMetrics}
		>
			{inner}
		</ModelLayout>
	};
	html.render_to_string()
}
