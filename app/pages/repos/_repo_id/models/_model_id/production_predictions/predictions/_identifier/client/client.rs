// use tangram_charts::{feature_contributions_chart::FeatureContributionsChart, hydrate_chart};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	// hydrate_chart::<FeatureContributionsChart>("regression_feature_contributions");
	// hydrate_chart::<FeatureContributionsChart>("binary_classification_feature_contributions");
	// hydrate_chart::<FeatureContributionsChart>("multiclass_classification_feature_contributions");
}
