use crate::{
	common::Point,
	config::{ChartColors, ChartConfig},
};
use html::style;
use wasm_bindgen::JsCast;
use web_sys::*;

pub struct DrawTooltipOptions<'a> {
	pub center_horizontal: Option<bool>,
	pub chart_colors: &'a ChartColors,
	pub chart_config: &'a ChartConfig,
	pub container: HtmlElement,
	pub flip_y_offset: Option<bool>,
	pub labels: Vec<TooltipLabel>,
	pub origin: Point,
}

pub struct TooltipLabel {
	pub color: String,
	pub text: String,
}

pub fn draw_tooltip(options: DrawTooltipOptions) {
	let DrawTooltipOptions {
		center_horizontal,
		chart_colors,
		chart_config,
		container,
		labels,
		origin: Point { x, y },
		..
	} = options;
	let center_horizontal = center_horizontal.unwrap_or(false);
	let window = window().unwrap();
	let document = window.document().unwrap();
	let tooltip_wrapper = document
		.create_element("div")
		.unwrap()
		.dyn_into::<HtmlElement>()
		.unwrap();
	let box_shadow = format!(
		"0 0 {} {}",
		chart_config.tooltip_shadow_blur, chart_colors.tooltip_shadow_color
	);
	let left = if center_horizontal {
		format!("{}px", x)
	} else {
		format!("calc({}px + 8px", x)
	};
	let transform = if center_horizontal {
		"translateX(-50%) translateY(-100%)"
	} else {
		"translateY(-100%)"
	};
	tooltip_wrapper.style().set_css_text(&style! {
		  "align-items" => "center",
		  "background-color" => chart_colors.tooltip_background_color,
		  "border-radius" => format!("{}px", chart_config.tooltip_border_radius),
		  "box-shadow" => box_shadow,
		  "display" => "grid",
		  "font" => chart_config.font,
		  "grid" => "auto / auto auto",
		  "grid-gap" => "0.5rem",
		  "left" => left,
		  "padding" => format!("{}px", chart_config.tooltip_padding),
		  "position" => "relative",
		  "top" => format!("calc({}px - 8px)", y),
		  "transform" => transform,
		  "user-select" => "none",
		  "width" => "max-content",
		  "z-index" => "2",
	});
	for label in labels {
		let tooltip_rect = document
			.create_element("div")
			.unwrap()
			.dyn_into::<HtmlElement>()
			.unwrap();
		tooltip_rect.style().set_css_text(&style! {
			"background-color" => label.color,
			"border-radius" => format!("{}px", chart_config.tooltip_border_radius),
			"height" => format!("{}px", chart_config.font_size),
			"width" => format!("{}px", chart_config.font_size),
		});
		let tooltip_label = document
			.create_element("div")
			.unwrap()
			.dyn_into::<HtmlElement>()
			.unwrap();
		tooltip_label.set_inner_text(&label.text);
		tooltip_wrapper.append_child(&tooltip_rect).unwrap();
		tooltip_wrapper.append_child(&tooltip_label).unwrap();
	}
	container.append_child(&tooltip_wrapper).unwrap();
	// // If the tooltip is not visible, place it elsewhere.
	// let bounding_rect = tooltip_wrapper.get_bounding_client_rect();
	// let window_width = window.inner_width().unwrap().as_f64().unwrap();
	// let overflow_right = bounding_rect.x() + bounding_rect.width() - window_width;
	// let overflow_left = -bounding_rect.x();
	// let padding = "16px";
	// // Translate by the amount that it is overflowing.
	// if overflow_right > 0.0 {
	// 	let transform = format!(
	// 		"translateX(calc(-50% - {}px - {})) translateY(-100%)",
	// 		overflow_right, padding
	// 	);
	// 	tooltip_wrapper
	// 		.style()
	// 		.set_property("transform", &transform)
	// 		.unwrap();
	// } else if overflow_left > 0.0 {
	// 	let transform = format!(
	// 		"translateX(calc(-50% + {}px + {})) translateY(-100%)",
	// 		overflow_left, padding
	// 	);
	// 	tooltip_wrapper
	// 		.style()
	// 		.set_property("transform", &transform)
	// 		.unwrap();
	// }
}
