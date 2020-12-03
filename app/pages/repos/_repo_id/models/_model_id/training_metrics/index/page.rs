use crate::binary_classifier::BinaryClassifierTrainingMetricsIndexPage;
use crate::multiclass_classifier::MulticlassClassifierTrainingMetricsIndexPage;
use crate::regressor::RegressorTrainingMetricsIndexPage;
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_deps::html::{self, html};

pub struct Props {
	pub id: String,
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

pub enum Inner {
	Regressor(RegressorProps),
	BinaryClassifier(BinaryClassifierProps),
	MulticlassClassifier(MulticlassClassifierProps),
}

#[derive(Clone)]
pub struct RegressorProps {
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
	pub mse: f32,
	pub rmse: f32,
	pub id: String,
}

#[derive(Clone)]
pub struct BinaryClassifierProps {
	pub accuracy: f32,
	pub baseline_accuracy: f32,
	pub auc_roc: f32,
	pub id: String,
	pub precision: f32,
	pub recall: f32,
	pub f1_score: f32,
}

#[derive(Clone)]
pub struct MulticlassClassifierProps {
	pub accuracy: f32,
	pub baseline_accuracy: f32,
	pub class_metrics: Vec<ClassMetrics>,
	pub classes: Vec<String>,
	pub id: String,
}

#[derive(Clone)]
pub struct ClassMetrics {
	pub precision: f32,
	pub recall: f32,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let inner = match props.inner {
		Inner::Regressor(inner) => {
			html! {<RegressorTrainingMetricsIndexPage props={inner} />}
		}
		Inner::BinaryClassifier(inner) => {
			html! {<BinaryClassifierTrainingMetricsIndexPage props={inner} />}
		}
		Inner::MulticlassClassifier(inner) => {
			html! {<MulticlassClassifierTrainingMetricsIndexPage props={inner} />}
		}
	};
	let html = html! {
		<ModelLayout
			info={props.model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::TrainingMetrics}
		>
			{inner}
		</ModelLayout>
	};
	html.render_to_string()
}
