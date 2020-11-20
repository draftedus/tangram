use crate::{
	chart::create_chart,
	common::{ChartBox, GridLineInterval},
	// config::{ChartColors, CHART_COLORS, CHART_CONFIG, DARK_CHART_COLORS, LIGHT_CHART_COLORS},
};
use std::cmp::Ordering;
// use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
// use web_sys::console;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BarChartOptions {
	pub data: BarChartData,
	pub class: Option<String>,
	pub id: Option<String>,
	pub title: Option<String>,
	pub group_gap: Option<usize>,
	pub hide_legend: Option<bool>,
	pub should_draw_x_axis_labels: Option<bool>,
	pub should_draw_y_axis_labels: Option<bool>,
	pub x_axis_title: Option<String>,
	pub y_axis_grid_line_interval: Option<GridLineInterval>,
	pub y_axis_title: Option<String>,
	pub y_max: Option<f32>,
	pub y_min: Option<f32>,
}

pub type BarChartData = Vec<BarChartSeries>;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct BarChartSeries {
	color: String,
	data: Vec<BarChartPoint>,
	title: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct BarChartPoint {
	label: String,
	x: f32,
	y: Option<f32>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BarChartOverlayInfo {
	chart_box: ChartBox,
}

// pub struct BarChartHoverRegionInfo {
// 	chart_box: ChartBox,
// 	color: String,
// 	point: BarChartPoint,
// 	tooltip_origin_pixels: Point,
// }

// pub struct DrawBarChartOutput {
// 	hover_regions: Vec<HoverRegion<BarChartHoverRegionInfo>>,
// 	overlay_info: BarChartOverlayInfo,
// }

pub fn create_bar_chart(container: web_sys::HtmlElement) -> web_sys::HtmlCanvasElement {
	create_chart(container)
}

pub fn draw_bar_chart(ctx: web_sys::CanvasRenderingContext2d, options: BarChartOptions) {
	let _width = ctx.canvas().unwrap().client_width();
	let _height = ctx.canvas().unwrap().client_height();
	// let hover_regions: Vec<HoverRegion<BarChartHoverRegionInfo>> = Vec::new();
	let y_min = options.y_min.unwrap_or_else(|| {
		0.0f32.min(
			options
				.data
				.iter()
				.flat_map(|series| series.data.iter().map(|p| p.y.unwrap_or(f32::INFINITY)))
				.min_by(|a, b| a.partial_cmp(b).unwrap())
				.unwrap_or(0.0),
		)
	});
	let mut _y_max = options.y_max.unwrap_or_else(|| {
		0.0f32.max(
			options
				.data
				.iter()
				.flat_map(|series| series.data.iter().map(|p| p.y.unwrap_or(f32::NEG_INFINITY)))
				.max_by(|a, b| a.partial_cmp(b).unwrap())
				.unwrap_or(0.0),
		)
	});
	if let Some(Ordering::Equal) = _y_max.partial_cmp(&y_min) {
		_y_max = y_min + 1.0;
	}
}

pub fn hydrate_bar_chart(id: &str) {
	let document = web_sys::window().unwrap().document().unwrap();
	let container = document
		.get_element_by_id(&id)
		.unwrap()
		.dyn_into::<web_sys::HtmlElement>()
		.unwrap();
	let options = container.dataset().get("options").unwrap();
	let options = serde_json::from_str(&options).unwrap();
	let canvas = create_bar_chart(container);
	let ctx = canvas
		.get_context("2d")
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::CanvasRenderingContext2d>()
		.unwrap();
	draw_bar_chart(ctx, options);
}
