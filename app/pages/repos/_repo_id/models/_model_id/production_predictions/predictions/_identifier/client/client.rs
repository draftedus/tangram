use tangram_charts::{feature_contributions_chart::FeatureContributionsChart, hydrate_chart};
use wasm_bindgen::prelude::*;
use web_sys::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	let window = window().unwrap();
	let document = window.document().unwrap();
	if document
		.get_element_by_id("regression_feature_contributions")
		.is_some()
	{
		hydrate_chart::<FeatureContributionsChart>("regression_feature_contributions");
	}
	if document
		.get_element_by_id("binary_classification_feature_contributions")
		.is_some()
	{
		hydrate_chart::<FeatureContributionsChart>("binary_classification_feature_contributions");
	}
	if document
		.get_element_by_id("multiclass_classification_feature_contributions")
		.is_some()
	{
		hydrate_chart::<FeatureContributionsChart>(
			"multiclass_classification_feature_contributions",
		);
	}
}
