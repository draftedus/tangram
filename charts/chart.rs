use crate::config::{CHART_CONFIG, DARK_CHART_COLORS, LIGHT_CHART_COLORS};
use num_traits::ToPrimitive;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

struct Chart;

impl Chart {}

impl<T> ChartTrait<T> for Chart {
	fn destroy(&self) {}
	fn draw(&self, options: T) {}
}

pub trait ChartTrait<T> {
	fn destroy(&self);
	fn draw(&self, options: T);
}

pub trait DrawFunction<Options, Info, HoverRegionInfo>:
	Fn(web_sys::CanvasRenderingContext2d, Options) -> DrawOutput<Info, HoverRegionInfo>
{
}

pub struct DrawOutput<OverlayInfo, HoverRegionInfo> {
	hover_regions: Vec<HoverRegion<HoverRegionInfo>>,
	overlay_info: OverlayInfo,
}

pub trait DrawOverlayFunction<Info, HoverRegionInfo>:
	Fn(DrawOverlayOptions<Info, HoverRegionInfo>)
{
}

pub struct DrawOverlayOptions<Info, HoverRegionInfo> {
	active_hover_regions: Vec<ActiveHoverRegion<HoverRegionInfo>>,
	ctx: web_sys::CanvasRenderingContext2d,
	info: Info,
	overlay_div: web_sys::HtmlElement,
}

pub struct HoverRegion<HoverRegionInfo> {
	info: HoverRegionInfo,
}

impl<HoverRegionInfo> HoverRegionTrait for HoverRegion<HoverRegionInfo> {
	fn distance(&self, x: i32, y: i32) -> f32 {
		todo!()
	}
	fn hit_test(&self, x: i32, y: i32) -> bool {
		todo!()
	}
}

trait HoverRegionTrait {
	fn distance(&self, x: i32, y: i32) -> f32;
	fn hit_test(&self, x: i32, y: i32) -> bool;
}

pub struct ActiveHoverRegion<HoverRegionInfo> {
	distance: f32,
	info: HoverRegionInfo,
}

pub struct ChartState<Options, OverlayInfo, HoverRegionInfo> {
	active_hover_regions: Option<Vec<ActiveHoverRegion<HoverRegionInfo>>>,
	hover_regions: Option<Vec<HoverRegion<HoverRegionInfo>>>,
	options: Option<Options>,
	overlay_info: Option<OverlayInfo>,
}

pub fn create_chart<Options, OverlayInfo, HoverRegionInfo>(
	container: web_sys::HtmlElement,
	draw_chart: impl DrawFunction<Options, OverlayInfo, HoverRegionInfo>,
	draw_chart_overlay: Option<impl DrawOverlayFunction<OverlayInfo, HoverRegionInfo>>,
) -> Box<dyn ChartTrait<Options>> {
	todo!()
	// let state: ChartState<Options, OverlayInfo, HoverRegionInfo> = ChartState {
	// 	active_hover_regions: None,
	// 	hover_regions: None,
	// 	options: None,
	// 	overlay_info: None,
	// };

	// let window = web_sys::window().unwrap();
	// let document = window.document().unwrap();

	// container
	// 	.style()
	// 	.set_property("position", "relative")
	// 	.unwrap();

	// let chart_canvas = document
	// 	.create_element("canvas")
	// 	.unwrap()
	// 	.dyn_into::<web_sys::HtmlCanvasElement>()
	// 	.unwrap();
	// chart_canvas
	// 	.style()
	// 	.set_property("position", "absolute")
	// 	.unwrap();
	// chart_canvas.style().set_property("top", "0").unwrap();
	// chart_canvas.style().set_property("bottom", "0").unwrap();
	// chart_canvas.style().set_property("left", "0").unwrap();
	// chart_canvas.style().set_property("right", "0").unwrap();
	// container.append_child(&chart_canvas).unwrap();

	// let overlay_div = document
	// 	.create_element("div")
	// 	.unwrap()
	// 	.dyn_into::<web_sys::HtmlElement>()
	// 	.unwrap();
	// overlay_div
	// 	.style()
	// 	.set_property("position", "absolute")
	// 	.unwrap();
	// overlay_div.style().set_property("top", "0").unwrap();
	// overlay_div.style().set_property("bottom", "0").unwrap();
	// overlay_div.style().set_property("left", "0").unwrap();
	// overlay_div.style().set_property("right", "0").unwrap();
	// container.append_child(&overlay_div).unwrap();

	// let overlay_canvas = document
	// 	.create_element("canvas")
	// 	.unwrap()
	// 	.dyn_into::<web_sys::HtmlCanvasElement>()
	// 	.unwrap();
	// overlay_canvas
	// 	.style()
	// 	.set_property("position", "absolute")
	// 	.unwrap();
	// overlay_canvas.style().set_property("top", "0").unwrap();
	// overlay_canvas.style().set_property("bottom", "0").unwrap();
	// overlay_canvas.style().set_property("left", "0").unwrap();
	// overlay_canvas.style().set_property("right", "0").unwrap();
	// container.append_child(&overlay_canvas).unwrap();

	// fn render_chart_overlay<Options, OverlayInfo, HoverRegionInfo>(
	// 	state: ChartState<Options, OverlayInfo, HoverRegionInfo>,
	// 	container: web_sys::HtmlElement,
	// 	overlay_canvas: web_sys::HtmlCanvasElement,
	// 	chart_canvas: web_sys::HtmlCanvasElement,
	// 	overlay_div: web_sys::HtmlElement,
	// 	draw_chart_overlay: Option<impl DrawOverlayFunction<OverlayInfo, HoverRegionInfo>>,
	// ) {
	// 	let window = web_sys::window().unwrap();
	// 	let overlay_info = state.overlay_info.unwrap();
	// 	let width = container.client_width().to_f64().unwrap();
	// 	let height = container.client_height().to_f64().unwrap();
	// 	let dpr = window.device_pixel_ratio();
	// 	overlay_canvas.set_width((width * dpr).to_u32().unwrap());
	// 	overlay_canvas.set_height((height * dpr).to_u32().unwrap());
	// 	overlay_canvas
	// 		.style()
	// 		.set_property("border", "solid")
	// 		.unwrap();
	// 	overlay_canvas
	// 		.style()
	// 		.set_property("width", &format!("{}px", width));
	// 	overlay_canvas
	// 		.style()
	// 		.set_property("height", &format!("{}px", height));
	// 	// chart_colors.current = color_scheme_media_query.matches
	// 	// 	? darkChartColors
	// 	// 	: lightChartColors
	// 	let ctx = chart_canvas
	// 		.get_context("2d")
	// 		.unwrap()
	// 		.unwrap()
	// 		.dyn_into::<web_sys::CanvasRenderingContext2d>()
	// 		.unwrap();
	// 	ctx.scale(dpr, dpr);
	// 	ctx.clear_rect(0.0, 0.0, width, height);
	// 	let child_nodes = overlay_div.child_nodes();
	// 	for child_node_index in 0..child_nodes.length() {
	// 		let child_node = child_nodes.item(child_node_index).unwrap();
	// 		overlay_div.remove_child(&child_node);
	// 	}
	// 	ctx.set_font(CHART_CONFIG.font);
	// 	draw_chart_overlay.map(|draw_chart_overlay| {
	// 		draw_chart_overlay(DrawOverlayOptions {
	// 			active_hover_regions: state.active_hover_regions.unwrap(),
	// 			ctx,
	// 			overlay_div,
	// 			info: state.overlay_info.unwrap(),
	// 		});
	// 	});
	// };

	// fn render_chart<Options, OverlayInfo, HoverRegionInfo>(
	// 	state: ChartState<Options, OverlayInfo, HoverRegionInfo>,
	// 	container: web_sys::HtmlElement,
	// 	overlay_canvas: web_sys::HtmlCanvasElement,
	// 	chart_canvas: web_sys::HtmlCanvasElement,
	// 	overlay_div: web_sys::HtmlElement,
	// 	draw_chart: impl DrawFunction<Options, OverlayInfo, HoverRegionInfo>,
	// ) {
	// 	let window = web_sys::window().unwrap();
	// 	let options = state.options.unwrap();
	// 	let width = container.client_width().to_f64().unwrap();
	// 	let height = container.client_height().to_f64().unwrap();
	// 	let dpr = window.device_pixel_ratio();
	// 	chart_canvas.set_width((width * dpr).to_u32().unwrap());
	// 	chart_canvas.set_height((height * dpr).to_u32().unwrap());
	// 	chart_canvas
	// 		.style()
	// 		.set_property("border", "solid")
	// 		.unwrap();
	// 	chart_canvas
	// 		.style()
	// 		.set_property("width", &format!("{}px", width));
	// 	chart_canvas
	// 		.style()
	// 		.set_property("height", &format!("{}px", height));
	// 	// chart_colors.current = colorSchemeMediaQuery.matches
	// 	// 	? darkChartColors
	// 	// 	: lightChartColors
	// 	let ctx = chart_canvas
	// 		.get_context("2d")
	// 		.unwrap()
	// 		.unwrap()
	// 		.dyn_into::<web_sys::CanvasRenderingContext2d>()
	// 		.unwrap();
	// 	ctx.scale(dpr, dpr);
	// 	ctx.clear_rect(0.0, 0.0, width, height);
	// 	ctx.set_font(CHART_CONFIG.font);
	// 	let output = draw_chart(ctx, state.options.unwrap());
	// 	state.hover_regions = Some(output.hover_regions);
	// 	state.overlay_info = Some(output.overlay_info);
	// };

	// let render = || {};

	// let update_active_hover_regions = |x: i32, y: i32| {
	// 	let hover_regions = state.hover_regions.unwrap();
	// 	let active_hover_regions = vec![];
	// 	for hover_region in hover_regions.iter() {
	// 		if hover_region.hit_test(x, y) {
	// 			active_hover_regions.push(ActiveHoverRegion {
	// 				distance: hover_region.distance(x, y),
	// 				info: hover_region.info,
	// 			})
	// 		}
	// 	}
	// 	state.active_hover_regions = Some(active_hover_regions);
	// 	render_chart_overlay(
	// 		state,
	// 		container,
	// 		overlay_canvas,
	// 		chart_canvas,
	// 		overlay_div,
	// 		draw_chart_overlay,
	// 	);
	// };

	// let on_mouse_event = |event: web_sys::MouseEvent| {
	// 	let canvas_client_rect = chart_canvas.get_bounding_client_rect();
	// 	let x = event.client_x() - canvas_client_rect.left().to_i32().unwrap();
	// 	let y = event.client_y() - canvas_client_rect.top().to_i32().unwrap();
	// 	update_active_hover_regions(x, y);
	// };
	// let on_mouse_event = Closure::wrap(Box::new(on_mouse_event) as Box<dyn FnMut(_)>);
	// overlay_canvas
	// 	.add_event_listener_with_callback("mouseenter", on_mouse_event.as_ref().unchecked_ref());
	// overlay_canvas
	// 	.add_event_listener_with_callback("mouseleave", on_mouse_event.as_ref().unchecked_ref());
	// overlay_canvas
	// 	.add_event_listener_with_callback("mousemove", on_mouse_event.as_ref().unchecked_ref());

	// let on_touch_event = |event: &web_sys::TouchEvent| {
	// 	let canvas_client_rect = chart_canvas.get_bounding_client_rect();
	// 	let x = event.touches().item(0).unwrap().client_x()
	// 		- canvas_client_rect.left().to_i32().unwrap();
	// 	let y = event.touches().item(0).unwrap().client_y()
	// 		- canvas_client_rect.top().to_i32().unwrap();
	// 	update_active_hover_regions(x, y);
	// };
	// let on_touch_event = Closure::wrap(Box::new(on_touch_event) as Box<dyn FnMut(_)>);
	// overlay_canvas
	// .add_event_listener_with_callback("touchstart", on_touch_event.as_ref().unchecked_ref());

	// let render_closure = Closure::wrap(Box::new(move || {
	// 	render_chart(
	// 		state,
	// 		container,
	// 		overlay_canvas,
	// 		chart_canvas,
	// 		overlay_div,
	// 		draw_chart,
	// 	);
	// 	render_chart_overlay(
	// 		state,
	// 		container,
	// 		overlay_canvas,
	// 		chart_canvas,
	// 		overlay_div,
	// 		draw_chart_overlay,
	// 	);
	// }) as Box<dyn FnMut()>);
	// window.add_event_listener_with_callback("resize", render_closure.as_ref().unchecked_ref());

	// let color_scheme_media_query = window
	// 	.match_media("(prefers-color-scheme: dark)")
	// 	.unwrap()
	// 	.unwrap();
	// color_scheme_media_query
	// 	.add_event_listener_with_callback("change", render_closure.as_ref().unchecked_ref());
	// let dpr = window.device_pixel_ratio();
	// let dpr_media_query = window
	// 	.match_media(&format!("(resolution: {}dppx)", dpr))
	// 	.unwrap()
	// 	.unwrap();
	// dpr_media_query
	// 	.add_event_listener_with_callback("change", render_closure.as_ref().unchecked_ref());

	// let destroy = || {
	// 	window
	// 		.remove_event_listener_with_callback("resize", render_closure.as_ref().unchecked_ref());
	// 	color_scheme_media_query
	// 		.remove_event_listener_with_callback("change", render_closure.as_ref().unchecked_ref());
	// 	dpr_media_query
	// 		.remove_event_listener_with_callback("change", render_closure.as_ref().unchecked_ref());
	// 	container.remove_child(&chart_canvas);
	// 	container.remove_child(&overlay_canvas);
	// };

	// let draw = |new_options: Options| {
	// 	state.options = Some(new_options);
	// 	render()
	// };

	// let chart = Chart;
	// Box::new(chart)
}
