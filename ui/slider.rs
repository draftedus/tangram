use html::{component, html, style};
use num_traits::ToPrimitive;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn Slider(id: Option<String>, max: f32, min: f32, value: usize) {
	let percent = ((value.to_f32().unwrap() - min) / (max - min)) * 100.0;
	let progress_style = style! {
	  "width" =>  format!("{}%", percent),
	};
	let tooltip_style = style! {
	  "margin-left" =>  format!("{}%", percent),
	};
	let value = value.to_string();
	html! {
		<div class={"slider-wrapper"}>
			<input
				id={id}
				class={"slider-range"}
				max={max.to_string()}
				min={min.to_string()}
				type={"range"}
			/>
			<div class="slider-progress" style={progress_style} />
			<div class="slider-tooltip" style={tooltip_style}>
				{value}
			</div>
		</div>
	}
}

pub fn boot_slider(id: &str) {
	let document = web_sys::window().unwrap().document().unwrap();
	let slider = document.get_element_by_id(id).unwrap();
	let callback_fn = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: web_sys::Event| {
		if let Some(current_target) = event.current_target() {
			let current_target = &current_target
				.dyn_into::<web_sys::HtmlInputElement>()
				.unwrap();
			let value = current_target.value();
			let min: f32 = current_target.min().parse().unwrap();
			let max: f32 = current_target.max().parse().unwrap();
			let percent = ((value.parse::<f32>().unwrap() - min) / (max - min)) * 100.0;
			let parent_element = current_target
				.parent_element()
				.unwrap()
				.dyn_into::<web_sys::HtmlElement>()
				.unwrap();
			let slider_progress = parent_element
				.get_elements_by_class_name("slider-progress")
				.item(0)
				.unwrap()
				.dyn_into::<web_sys::HtmlElement>()
				.unwrap();
			slider_progress
				.style()
				.set_property("width", &format!("{}%", &percent))
				.unwrap();
			let slider_tooltip = parent_element
				.get_elements_by_class_name("slider-tooltip")
				.item(0)
				.unwrap()
				.dyn_into::<web_sys::HtmlElement>()
				.unwrap();
			slider_tooltip
				.style()
				.set_property("margin-left", &format!("{}%", &percent))
				.unwrap();
		}
	}));
	if let Some(slider) = slider.dyn_ref::<web_sys::HtmlInputElement>() {
		slider
			.add_event_listener_with_callback("input", callback_fn.as_ref().unchecked_ref())
			.unwrap();
	}
	callback_fn.forget();
}
