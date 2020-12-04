// use tangram_charts::{bar_chart::BarChart, box_chart::BoxChart, hydrate_chart};
use wasm_bindgen::prelude::*;
use web_sys::window;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	// let window = window().unwrap();
	// let document = window.document().unwrap();
	// if document.get_element_by_id("enum_histogram").is_some() {
	// 	hydrate_chart::<BarChart>("enum_histogram");
	// }
	// if document.get_element_by_id("number_quantiles").is_some() {
	// 	hydrate_chart::<BoxChart>("number_quantiles");
	// }
	// if document.get_element_by_id("number_histogram").is_some() {
	// 	hydrate_chart::<BarChart>("number_histogram");
	// }
	// if document.get_element_by_id("token_histogram").is_some() {
	// 	hydrate_chart::<BarChart>("token_histogram");
	// }
}
