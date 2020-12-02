use tangram_charts::{bar_chart::BarChart, hydrate_chart};
use wasm_bindgen::prelude::*;
use web_sys::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	hydrate_chart::<BarChart>("enum_histogram");
}
