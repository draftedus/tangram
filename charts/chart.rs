// use crate::config::{
// 	ChartColors, CHART_COLORS, CHART_CONFIG, DARK_CHART_COLORS, LIGHT_CHART_COLORS,
// };
// use std::cell::{Cell, RefCell, RefMut};
// use std::rc::Rc;
// use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// pub trait ChartTrait<T> {
// 	fn destroy(&self);
// 	fn draw(&self, options: T);
// }

// pub trait DrawFunction<Options, Info, HoverRegionInfo>:
// 	Fn(web_sys::CanvasRenderingContext2d, Options) -> DrawOutput<Info, HoverRegionInfo>
// where
// 	Options: Clone,
// 	Info: Clone,
// 	HoverRegionInfo: Clone,
// {
// }

// pub struct DrawOutput<OverlayInfo, HoverRegionInfo> {
// 	hover_regions: Vec<HoverRegion<HoverRegionInfo>>,
// 	overlay_info: OverlayInfo,
// }

// pub trait DrawOverlayFunction<Info, HoverRegionInfo>:
// 	Fn(DrawOverlayOptions<Info, HoverRegionInfo>)
// {
// }

// pub struct DrawOverlayOptions<Info, HoverRegionInfo> {
// 	active_hover_regions: Vec<ActiveHoverRegion<HoverRegionInfo>>,
// 	ctx: web_sys::CanvasRenderingContext2d,
// 	info: Info,
// 	overlay_div: web_sys::HtmlElement,
// }

#[derive(Clone)]
pub struct HoverRegion<HoverRegionInfo> {
	info: HoverRegionInfo,
}

// impl<HoverRegionInfo> HoverRegionTrait for HoverRegion<HoverRegionInfo> {
// 	fn distance(&self, x: i32, y: i32) -> f32 {
// 		todo!()
// 	}
// 	fn hit_test(&self, x: i32, y: i32) -> bool {
// 		todo!()
// 	}
// }

// trait HoverRegionTrait {
// 	fn distance(&self, x: i32, y: i32) -> f32;
// 	fn hit_test(&self, x: i32, y: i32) -> bool;
// }

// #[derive(Clone)]
// pub struct ActiveHoverRegion<HoverRegionInfo> {
// 	distance: f32,
// 	info: HoverRegionInfo,
// }

// #[derive(Clone)]
// pub struct ChartState<Options, OverlayInfo, HoverRegionInfo> {
// 	active_hover_regions: Option<Vec<ActiveHoverRegion<HoverRegionInfo>>>,
// 	hover_regions: Option<Vec<HoverRegion<HoverRegionInfo>>>,
// 	options: Option<Options>,
// 	overlay_info: Option<OverlayInfo>,
// }

pub fn create_chart(container: web_sys::HtmlElement) -> web_sys::HtmlCanvasElement
// 	draw_chart: Box<dyn DrawFunction<Options, OverlayInfo, HoverRegionInfo>>,
// 	draw_chart_overlay: Option<impl DrawOverlayFunction<OverlayInfo, HoverRegionInfo>>,
// ) -> Box<dyn ChartTrait<Options>>
// where
// 	Options: Clone + 'static,
// 	HoverRegionInfo: Clone + 'static,
// 	OverlayInfo: Clone + 'static,
{
	// 	let state: ChartState<Options, OverlayInfo, HoverRegionInfo> = ChartState {
	// 		active_hover_regions: None,
	// 		hover_regions: None,
	// 		options: None,
	// 		overlay_info: None,
	// 	};

	// 	let mut current_chart_colors = Rc::new(Cell::new(CHART_COLORS));

	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();

	container
		.style()
		.set_property("position", "relative")
		.unwrap();

	let chart_canvas = document
		.create_element("canvas")
		.unwrap()
		.dyn_into::<web_sys::HtmlCanvasElement>()
		.unwrap();
	chart_canvas
		.style()
		.set_property("position", "absolute")
		.unwrap();
	chart_canvas.style().set_property("top", "0").unwrap();
	chart_canvas.style().set_property("bottom", "0").unwrap();
	chart_canvas.style().set_property("left", "0").unwrap();
	chart_canvas.style().set_property("right", "0").unwrap();
	container.append_child(&chart_canvas).unwrap();

	let overlay_div = document
		.create_element("div")
		.unwrap()
		.dyn_into::<web_sys::HtmlElement>()
		.unwrap();
	overlay_div
		.style()
		.set_property("position", "absolute")
		.unwrap();
	overlay_div.style().set_property("top", "0").unwrap();
	overlay_div.style().set_property("bottom", "0").unwrap();
	overlay_div.style().set_property("left", "0").unwrap();
	overlay_div.style().set_property("right", "0").unwrap();
	container.append_child(&overlay_div).unwrap();

	let overlay_canvas = document
		.create_element("canvas")
		.unwrap()
		.dyn_into::<web_sys::HtmlCanvasElement>()
		.unwrap();
	overlay_canvas
		.style()
		.set_property("position", "absolute")
		.unwrap();
	overlay_canvas.style().set_property("top", "0").unwrap();
	overlay_canvas.style().set_property("bottom", "0").unwrap();
	overlay_canvas.style().set_property("left", "0").unwrap();
	overlay_canvas.style().set_property("right", "0").unwrap();
	container.append_child(&overlay_canvas).unwrap();

	chart_canvas

	// 	let state = RefCell::new(state);
	// 	{
	// 		let container_ref = container.clone();
	// 		let overlay_canvas_ref = overlay_canvas.clone();
	// 		let overlay_div_ref = overlay_div.clone();
	// 		let state = state.clone();
	// 		let on_mouse_event =
	// 			Closure::<dyn FnMut(_)>::wrap(Box::new(move |event: web_sys::MouseEvent| {
	// 				let canvas_client_rect = chart_canvas.get_bounding_client_rect();
	// 				let x = event.client_x() - canvas_client_rect.left().to_i32().unwrap();
	// 				let y = event.client_y() - canvas_client_rect.top().to_i32().unwrap();
	// 				update_active_hover_regions(
	// 					state.borrow_mut(),
	// 					&container_ref,
	// 					&overlay_canvas_ref,
	// 					&chart_canvas,
	// 					&overlay_div_ref,
	// 					// draw_chart_overlay,
	// 					// current_chart_colors,
	// 					x,
	// 					y,
	// 				);
	// 			}));
	// 		overlay_canvas
	// 			.add_event_listener_with_callback("mouseenter", on_mouse_event.as_ref().unchecked_ref())
	// 			.unwrap();
	// 		overlay_canvas
	// 			.add_event_listener_with_callback("mouseleave", on_mouse_event.as_ref().unchecked_ref())
	// 			.unwrap();
	// 		overlay_canvas
	// 			.add_event_listener_with_callback("mousemove", on_mouse_event.as_ref().unchecked_ref())
	// 			.unwrap();
	// 		on_mouse_event.forget();
	// 	}

	// 	// let on_touch_event = |event: &web_sys::TouchEvent| {
	// 	// 	let canvas_client_rect = chart_canvas.get_bounding_client_rect();
	// 	// 	let x = event.touches().item(0).unwrap().client_x()
	// 	// 		- canvas_client_rect.left().to_i32().unwrap();
	// 	// 	let y = event.touches().item(0).unwrap().client_y()
	// 	// 		- canvas_client_rect.top().to_i32().unwrap();
	// 	// 	update_active_hover_regions(x, y);
	// 	// };
	// 	// let on_touch_event = Closure::wrap(Box::new(on_touch_event) as Box<dyn FnMut(_)>);
	// 	// overlay_canvas
	// 	// 	.add_event_listener_with_callback("touchstart", on_touch_event.as_ref().unchecked_ref());

	// 	// let render_closure = Closure::wrap(Box::new(move || {
	// 	// 	render_chart(
	// 	// 		state,
	// 	// 		container,
	// 	// 		overlay_canvas,
	// 	// 		chart_canvas,
	// 	// 		overlay_div,
	// 	// 		draw_chart,
	// 	// 	);
	// 	// 	render_chart_overlay(
	// 	// 		state,
	// 	// 		container,
	// 	// 		overlay_canvas,
	// 	// 		chart_canvas,
	// 	// 		overlay_div,
	// 	// 		draw_chart_overlay,
	// 	// 	);
	// 	// }) as Box<dyn FnMut()>);
	// 	// window.add_event_listener_with_callback("resize", render_closure.as_ref().unchecked_ref());

	// 	// let color_scheme_media_query = window
	// 	// 	.match_media("(prefers-color-scheme: dark)")
	// 	// 	.unwrap()
	// 	// 	.unwrap();
	// 	// color_scheme_media_query
	// 	// 	.add_event_listener_with_callback("change", render_closure.as_ref().unchecked_ref());
	// 	// let dpr = window.device_pixel_ratio();
	// 	// let dpr_media_query = window
	// 	// 	.match_media(&format!("(resolution: {}dppx)", dpr))
	// 	// 	.unwrap()
	// 	// 	.unwrap();
	// 	// dpr_media_query
	// 	// 	.add_event_listener_with_callback("change", render_closure.as_ref().unchecked_ref());

	// 	// let destroy = || {
	// 	// 	window
	// 	// 		.remove_event_listener_with_callback("resize", render_closure.as_ref().unchecked_ref());
	// 	// 	color_scheme_media_query
	// 	// 		.remove_event_listener_with_callback("change", render_closure.as_ref().unchecked_ref());
	// 	// 	dpr_media_query
	// 	// 		.remove_event_listener_with_callback("change", render_closure.as_ref().unchecked_ref());
	// 	// 	container.remove_child(&chart_canvas);
	// 	// 	container.remove_child(&overlay_canvas);
	// 	// };

	// 	// let draw = |new_options: Options| {
	// 	// 	state.options = Some(new_options);
	// 	// 	render()
	// 	// };
	// 	todo!()
}

// fn render_chart<'a, Options, OverlayInfo, HoverRegionInfo>(
// 	state: &mut ChartState<Options, OverlayInfo, HoverRegionInfo>,
// 	container: web_sys::HtmlElement,
// 	overlay_canvas: &web_sys::HtmlCanvasElement,
// 	chart_canvas: &web_sys::HtmlCanvasElement,
// 	overlay_div: &web_sys::HtmlElement,
// 	draw_chart: impl DrawFunction<Options, OverlayInfo, HoverRegionInfo>,
// 	chart_colors: &mut ChartColors,
// ) where
// 	Options: Clone,
// 	HoverRegionInfo: Clone,
// 	OverlayInfo: Clone,
// {
// 	let window = web_sys::window().unwrap();
// 	let options = state.options.as_ref().unwrap().clone();
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
// 		.set_property("width", &format!("{}px", width))
// 		.unwrap();
// 	chart_canvas
// 		.style()
// 		.set_property("height", &format!("{}px", height))
// 		.unwrap();
// 	let color_scheme_media_query = window
// 		.match_media("(prefers-color-scheme: dark)")
// 		.unwrap()
// 		.unwrap();
// 	*chart_colors = if color_scheme_media_query.matches() {
// 		DARK_CHART_COLORS
// 	} else {
// 		LIGHT_CHART_COLORS
// 	};
// 	let ctx = chart_canvas
// 		.get_context("2d")
// 		.unwrap()
// 		.unwrap()
// 		.dyn_into::<web_sys::CanvasRenderingContext2d>()
// 		.unwrap();
// 	ctx.scale(dpr, dpr).unwrap();
// 	ctx.clear_rect(0.0, 0.0, width, height);
// 	ctx.set_font(CHART_CONFIG.font);
// 	let output = draw_chart(ctx, options);
// 	state.hover_regions = Some(output.hover_regions);
// 	state.overlay_info = Some(output.overlay_info);
// }

// fn render_chart_overlay<Options, OverlayInfo, HoverRegionInfo>(
// 	state: RefMut<ChartState<Options, OverlayInfo, HoverRegionInfo>>,
// 	container: &web_sys::HtmlElement,
// 	overlay_canvas: &web_sys::HtmlCanvasElement,
// 	chart_canvas: &web_sys::HtmlCanvasElement,
// 	overlay_div: &web_sys::HtmlElement,
// 	// draw_chart_overlay: Option<impl DrawOverlayFunction<OverlayInfo, HoverRegionInfo>>,
// 	// chart_colors: &mut ChartColors,
// ) where
// 	HoverRegionInfo: Clone,
// 	OverlayInfo: Clone,
// 	Options: Clone,
// {
// 	let window = web_sys::window().unwrap();
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
// 		.set_property("width", &format!("{}px", width))
// 		.unwrap();
// 	overlay_canvas
// 		.style()
// 		.set_property("height", &format!("{}px", height))
// 		.unwrap();
// 	let color_scheme_media_query = window
// 		.match_media("(prefers-color-scheme: dark)")
// 		.unwrap()
// 		.unwrap();
// 	// *chart_colors = if color_scheme_media_query.matches() {
// 	// 	DARK_CHART_COLORS
// 	// } else {
// 	// 	LIGHT_CHART_COLORS
// 	// };
// 	let ctx = chart_canvas
// 		.get_context("2d")
// 		.unwrap()
// 		.unwrap()
// 		.dyn_into::<web_sys::CanvasRenderingContext2d>()
// 		.unwrap();
// 	ctx.scale(dpr, dpr).unwrap();
// 	ctx.clear_rect(0.0, 0.0, width, height);
// 	let child_nodes = overlay_div.child_nodes();
// 	for child_node_index in 0..child_nodes.length() {
// 		let child_node = child_nodes.item(child_node_index).unwrap();
// 		overlay_div.remove_child(&child_node).unwrap();
// 	}
// 	ctx.set_font(CHART_CONFIG.font);
// 	// (draw_chart_overlay.unwrap())(DrawOverlayOptions {
// 	// 	active_hover_regions: state.active_hover_regions.as_ref().unwrap().clone(),
// 	// 	ctx,
// 	// 	overlay_div: overlay_div.to_owned(),
// 	// 	info: state.overlay_info.as_ref().unwrap().clone(),
// 	// });
// }

// fn update_active_hover_regions<Options, OverlayInfo, HoverRegionInfo>(
// 	mut state: RefMut<ChartState<Options, OverlayInfo, HoverRegionInfo>>,
// 	container: &web_sys::HtmlElement,
// 	overlay_canvas: &web_sys::HtmlCanvasElement,
// 	chart_canvas: &web_sys::HtmlCanvasElement,
// 	overlay_div: &web_sys::HtmlElement,
// 	// draw_chart_overlay: Option<impl DrawOverlayFunction<OverlayInfo, HoverRegionInfo>>,
// 	// chart_colors: &mut ChartColors,
// 	x: i32,
// 	y: i32,
// ) where
// 	Options: Clone,
// 	OverlayInfo: Clone,
// 	HoverRegionInfo: Clone,
// {
// 	let mut active_hover_regions = vec![];
// 	for hover_region in state.hover_regions.as_ref().unwrap().iter() {
// 		if hover_region.hit_test(x, y) {
// 			active_hover_regions.push(ActiveHoverRegion {
// 				distance: hover_region.distance(x, y),
// 				info: hover_region.info.clone(),
// 			})
// 		}
// 	}
// 	state.active_hover_regions = Some(active_hover_regions);
// 	render_chart_overlay(
// 		state,
// 		&container,
// 		&overlay_canvas,
// 		&chart_canvas,
// 		&overlay_div,
// 		// draw_chart_overlay,
// 		// chart_colors,
// 	);
// }
