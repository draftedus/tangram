use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
	fn alert(message: &str);
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
	alert("hello, world!");
	Ok(())
}

/*

export function bootFileFields() {
	let fileInputElements = document.querySelectorAll('input[type=file]')
	fileInputElements.forEach(fileInputElement => {
		if (!(fileInputElement instanceof HTMLInputElement)) throw Error()
		updateFileInputElement(fileInputElement)
		fileInputElement.addEventListener('change', () =>
			updateFileInputElement(fileInputElement),
		)
	})
}

function updateFileInputElement(fileInputElement: HTMLInputElement) {
	let file = fileInputElement.files?.item(0)
	if (file) {
		fileInputElement.parentElement?.firstChild?.replaceWith(
			document.createTextNode(file.name),
		)
	}
}

*/
