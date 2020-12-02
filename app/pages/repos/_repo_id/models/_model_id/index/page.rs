use crate::{
	binary_classifier::binary_classifier_index_page,
	multiclass_classifier::multiclass_classifier_index_page, regressor::regressor_index_page,
};
use html::html;
use tangram_app_layouts::document::PageInfo;
use tangram_app_layouts::model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem};

pub use {
	crate::binary_classifier::{BinaryClassifierInnerMetrics, BinaryClassifierProps},
	crate::multiclass_classifier::{
		MulticlassClassifierInnerClassMetrics, MulticlassClassifierInnerMetrics,
		MulticlassClassifierProps,
	},
	crate::regressor::{RegressorInnerMetrics, RegressorProps},
};

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

pub struct TrainingSummary {
	pub chosen_model_type_name: String,
	pub column_count: usize,
	pub model_comparison_metric_type_name: String,
	pub train_row_count: usize,
	pub test_row_count: usize,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let inner = match props.inner {
		Inner::Regressor(inner) => regressor_index_page(inner),
		Inner::BinaryClassifier(inner) => binary_classifier_index_page(inner),
		Inner::MulticlassClassifier(inner) => multiclass_classifier_index_page(inner),
	};
	let html = html! {
	<ModelLayout info={props.model_layout_info} page_info={page_info}selected_item={ModelSideNavItem::Overview}>
		{inner}
	</ModelLayout>
	};
	html.render_to_string()
}
