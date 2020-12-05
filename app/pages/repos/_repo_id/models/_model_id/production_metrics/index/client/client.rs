// use tangram_app_common::date_window_select_field::boot_date_window_select_field;
use tangram_charts::{hydrate_chart, line_chart::LineChart};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	// boot_date_window_select_field();
	hydrate_chart::<LineChart>("mse");
	hydrate_chart::<LineChart>("accuracy");
}
