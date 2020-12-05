use tangram_app_common::date_window_select_field::boot_date_window_select_field;
use tangram_charts::{bar_chart::BarChart, box_chart::BoxChart, hydrate_chart};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	boot_date_window_select_field();
	hydrate_chart::<BoxChart>("number_intervals");
	hydrate_chart::<BoxChart>("number_overall");
	hydrate_chart::<BarChart>("enum_overall");
	hydrate_chart::<BarChart>("text_overall");
}
