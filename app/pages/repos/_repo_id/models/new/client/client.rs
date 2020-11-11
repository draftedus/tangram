use wasm_bindgen::prelude::*;
use web_sys::window;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
	window()
		.unwrap()
		.alert_with_message("hello from rust!!!")
		.unwrap();
	Ok(())
}
