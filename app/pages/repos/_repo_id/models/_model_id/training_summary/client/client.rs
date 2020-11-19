use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
	console_error_panic_hook::set_once();
	tangram_charts::hydrate_bar_chart("bar-chart");
	Ok(())
}
