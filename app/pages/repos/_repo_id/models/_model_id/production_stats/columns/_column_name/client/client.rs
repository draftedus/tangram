use tangram_charts::{bar_chart::BarChart, box_chart::BoxChart, hydrate_chart};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	// TODO: boot date window select
	hydrate_chart::<BoxChart>("number_intervals");
	hydrate_chart::<BoxChart>("number_overall");
	hydrate_chart::<BarChart>("enum_overall");
	hydrate_chart::<BarChart>("text_overall");
}
