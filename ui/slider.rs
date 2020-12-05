use html::{component, html, style};
use num_traits::ToPrimitive;

#[component]
pub fn Slider(
	id: Option<String>,
	max: f32,
	min: f32,
	value: usize,
) {
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
