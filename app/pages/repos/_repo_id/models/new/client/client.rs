use tangram_ui as ui;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
	console_error_panic_hook::set_once();
	ui::boot_file_fields();
	Ok(())
}
