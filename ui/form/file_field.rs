use super::FieldLabel;
use html::{component, html};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn FileField(
	disabled: Option<bool>,
	label: Option<String>,
	name: Option<String>,
	required: Option<bool>,
) {
	html! {
		<FieldLabel html_for={None}>
			{label}
			<div class="form-file-wrapper">
				{"Choose File"}
				<input
					class="form-file-input"
					name={name}
					required={required}
					disabled={disabled}
					type="file"
				/>
			</div>
		</FieldLabel>
	}
}

/** When using a custom 'Choose File' prompt, it is necessary to use JS to update it to show the selected file name. */
pub fn boot_file_fields() {
	let document = web_sys::window().unwrap().document().unwrap();
	let file_input_elements = document.query_selector_all("input[type=file]").unwrap();
	for file_input_element_index in 0..file_input_elements.length() {
		let file_input_element = file_input_elements.item(file_input_element_index).unwrap();
		update_file_input_element(&file_input_element.clone());
		let file_input_element_for_closure = file_input_element.clone();
		let callback_fn = Closure::wrap(Box::new(move || {
			update_file_input_element(&file_input_element_for_closure)
		}) as Box<dyn FnMut()>);
		file_input_element
			.add_event_listener_with_callback("change", callback_fn.as_ref().unchecked_ref())
			.unwrap();
		callback_fn.forget();
		fn update_file_input_element(file_input_element: &web_sys::EventTarget) {
			let file = file_input_element
				.dyn_ref::<web_sys::HtmlInputElement>()
				.unwrap()
				.files()
				.and_then(|files| files.item(0));
			if let Some(file) = file {
				let file_name = file.name();
				if let Some(file_input_element) = file_input_element
					.dyn_ref::<web_sys::HtmlInputElement>()
					.and_then(|element| element.parent_element())
				{
					file_input_element
						.first_child()
						.unwrap()
						.set_text_content(Some(&file_name));
				};
			}
		}
	}
}
