use html::{component, html, style};

#[component]
pub fn Slider(max: f32, min: f32, value: f32, value_formatter: Option<fn(f32) -> String>) {
	let percent = ((value - min) / (max - min)) * 100.0;
	let progress_style = style! {
	  "width" =>  format!("{}%", percent),
	};
	let tooltip_style = style! {
	  "margin-left" =>  format!("{}%", percent),
	};
	let value = value_formatter
		.map(|value_formatter| value_formatter(value))
		.unwrap_or_else(|| value.to_string());
	html! {
		<div class={"slider-wrapper".to_owned()}>
			<input
				class={"slider-range".to_owned()}
				max={max.to_string()}
				min={min.to_string()}
				type={"range".to_owned()}
			/>
			<div class="slider-progress" style={progress_style} />
			<div class="slider-tooltip" style={tooltip_style}>
				{value}
			</div>
		</div>
	}
}
