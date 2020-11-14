use wasm_bindgen::prelude::*;
use web_sys::window;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
	let document = window().unwrap().document().unwrap();
	let element = document.create_element("p").unwrap();
	element.set_inner_html("hello from rust!!!");
	document.body().unwrap().append_child(&element).unwrap();
	Ok(())
}
