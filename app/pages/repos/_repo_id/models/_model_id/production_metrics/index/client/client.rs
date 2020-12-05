use tangram_charts::{hydrate_chart, line_chart::LineChart};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	// TODO: boot date window select
	hydrate_chart::<LineChart>("mse");
	hydrate_chart::<LineChart>("accuracy");
}
