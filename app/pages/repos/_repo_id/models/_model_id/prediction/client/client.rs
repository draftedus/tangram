use tangram_charts::{
	hydrate_chart,
	{
		bar_chart::BarChart, box_chart::BoxChart,
		feature_contributions_chart::FeatureContributionsChart,
	},
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();
	let bar_charts_query = document
		.query_selector_all(".column-chart[data-chart-type='bar']")
		.unwrap();
	for index in 0..bar_charts_query.length() {
		let item = bar_charts_query
			.item(index)
			.unwrap()
			.dyn_into::<web_sys::Element>()
			.unwrap();
		hydrate_chart::<BoxChart>(&item.id());
	}
	let box_charts_query = document
		.query_selector_all(".column-chart[data-chart-type='box']")
		.unwrap();
	for index in 0..box_charts_query.length() {
		let item = box_charts_query
			.item(index)
			.unwrap()
			.dyn_into::<web_sys::Element>()
			.unwrap();
		hydrate_chart::<BoxChart>(&item.id());
	}

	// hydrate_chart::<FeatureContributionsChart>("regression_feature_contributions")
	// hydrate_chart::<FeatureContributionsChart>("binary_classification_feature_contributions")
	// hydrate_chart::<FeatureContributionsChart>("multiclass_classification_feature_contributions")
}
