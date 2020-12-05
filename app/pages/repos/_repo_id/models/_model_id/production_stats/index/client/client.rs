use tangram_charts::{bar_chart::BarChart, box_chart::BoxChart, hydrate_chart};
use wasm_bindgen::prelude::*;
use tangram_ui as ui;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	ui::select_field_submit_on_change("date-window-select-field".to_owned());
	hydrate_chart::<BarChart>("prediction_count");
	hydrate_chart::<BoxChart>("quantiles_overall");
	hydrate_chart::<BoxChart>("quantiles_intervals");
	hydrate_chart::<BarChart>("histogram_overall");
	hydrate_chart::<BarChart>("histogram_intervals");
}
