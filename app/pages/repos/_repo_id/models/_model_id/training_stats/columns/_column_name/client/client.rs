use tangram_charts::{bar_chart::BarChart, box_chart::BoxChart, hydrate_chart};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	hydrate_chart::<BarChart>("enum_histogram");
	hydrate_chart::<BoxChart>("number_quantiles");
	hydrate_chart::<BarChart>("number_histogram");
	hydrate_chart::<BarChart>("token_histogram");
}
