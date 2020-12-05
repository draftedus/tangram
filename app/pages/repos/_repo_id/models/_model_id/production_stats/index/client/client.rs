// use tangram_app_common::date_window_select_field::boot_date_window_select_field;
use tangram_charts::{bar_chart::BarChart, box_chart::BoxChart, hydrate_chart};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	// boot_date_window_select_field();
	hydrate_chart::<BarChart>("prediction_count");
	hydrate_chart::<BoxChart>("quantiles_overall");
	hydrate_chart::<BoxChart>("quantiles_intervals");
	hydrate_chart::<BarChart>("histogram_overall");
	hydrate_chart::<BarChart>("histogram_intervals");
}
