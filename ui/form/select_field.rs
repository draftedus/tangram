use super::FieldLabel;
use html::{component, html};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone)]
pub struct SelectFieldOption {
	pub text: String,
	pub value: String,
}

#[component]
pub fn SelectField(
	disabled: Option<bool>,
	id: Option<String>,
	label: Option<String>,
	name: Option<String>,
	options: Vec<SelectFieldOption>,
	placeholder: Option<String>,
	required: Option<bool>,
	value: Option<String>,
) {
	html! {
		<FieldLabel html_for={None}>
			{label}
			<select
				class="form-select"
				disabled={disabled}
				id={id}
				name={name}
				placeholder={placeholder}
				required={required}
				value={value}
			>
				{
					options.iter().map(|option| {
						html! {
							<option key={option.value.clone()} value={option.value.clone()}>
								{option.text.clone()}
							</option>
						}
					}).collect::<Vec<_>>()
				}
			</select>
		</FieldLabel>
	}
}

pub fn select_field_submit_on_change(id: String) {
	let document = web_sys::window().unwrap().document().unwrap();
	let select_element = document.get_element_by_id(&id).unwrap();
	let callback_fn = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: web_sys::Event| {
		if let Some(event) = event.current_target() {
			let form = event
				.dyn_ref::<web_sys::HtmlElement>()
				.unwrap()
				.closest("form")
				.unwrap();
			form.unwrap()
				.dyn_ref::<web_sys::HtmlFormElement>()
				.unwrap()
				.submit()
				.ok();
		}
	}));
	if let Some(select_element) = select_element.dyn_ref::<web_sys::HtmlSelectElement>() {
		select_element
			.add_event_listener_with_callback("change", callback_fn.as_ref().unchecked_ref())
			.unwrap();
	}
	callback_fn.forget();
}
