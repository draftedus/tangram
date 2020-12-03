use crate::{
	bar_chart::{draw_bar_chart_x_axis_labels, DrawBarChartXAxisLabelsOptions},
	chart::{
		ActiveHoverRegion, ChartImpl, DrawChartOptions, DrawChartOutput, DrawOverlayOptions,
		HoverRegion,
	},
	common::{
		compute_boxes, draw_rounded_rect, draw_x_axis, draw_x_axis_title, draw_y_axis_grid_lines,
		draw_y_axis_labels, draw_y_axis_title, format_number, ComputeBoxesOptions,
		ComputeBoxesOutput, DrawRoundedRectOptions, DrawXAxisOptions, DrawXAxisTitleOptions,
		DrawYAxisGridLinesOptions, DrawYAxisLabelsOptions, DrawYAxisTitleOptions, GridLineInterval,
		Point, Rect,
	},
	config::ChartConfig,
};
use num_traits::ToPrimitive;
use wasm_bindgen::JsValue;
use web_sys::*;

pub struct BoxChart;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BoxChartOptions {
	pub hide_legend: Option<bool>,
	pub series: Vec<BoxChartSeries>,
	pub should_draw_x_axis_labels: Option<bool>,
	pub should_draw_y_axis_labels: Option<bool>,
	pub title: Option<String>,
	pub x_axis_title: Option<String>,
	pub y_axis_title: Option<String>,
	pub y_max: Option<f64>,
	pub y_min: Option<f64>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BoxChartSeries {
	pub color: String,
	pub data: Vec<BoxChartPoint>,
	pub title: Option<String>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BoxChartPoint {
	pub label: String,
	pub x: f64,
	pub y: Option<BoxChartValue>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BoxChartValue {
	pub max: f64,
	pub min: f64,
	pub p25: f64,
	pub p50: f64,
	pub p75: f64,
}

pub struct BoxChartOverlayInfo {
	chart_box: Rect,
}

#[derive(Clone)]
pub struct BoxChartHoverRegionInfo {
	color: String,
	label: String,
	name: String,
	tooltip_origin_pixels: Point,
	value: f64,
}

impl ChartImpl for BoxChart {
	type Options = BoxChartOptions;
	type OverlayInfo = BoxChartOverlayInfo;
	type HoverRegionInfo = BoxChartHoverRegionInfo;

	fn draw_chart(
		options: DrawChartOptions<Self::Options>,
	) -> DrawChartOutput<Self::OverlayInfo, Self::HoverRegionInfo> {
		draw_box_chart(options)
	}

	fn draw_overlay(options: DrawOverlayOptions<Self::OverlayInfo, Self::HoverRegionInfo>) {
		draw_box_chart_overlay(options)
	}
}

fn draw_box_chart(
	options: DrawChartOptions<BoxChartOptions>,
) -> DrawChartOutput<BoxChartOverlayInfo, BoxChartHoverRegionInfo> {
	let DrawChartOptions {
		chart_colors,
		chart_config,
		ctx,
		options,
	} = options;
	let BoxChartOptions {
		series,
		x_axis_title,
		y_axis_title,
		..
	} = &options;
	let canvas = ctx.canvas().unwrap();
	let width = canvas.client_width().to_f64().unwrap();
	let height = canvas.client_height().to_f64().unwrap();
	let mut hover_regions: Vec<HoverRegion<BoxChartHoverRegionInfo>> = Vec::new();

	// Compute the bounds.
	let y_min = options.y_min.unwrap_or_else(|| {
		series
			.iter()
			.flat_map(|series| {
				series
					.data
					.iter()
					.map(|p| p.y.as_ref().map(|y| y.min).unwrap_or(f64::INFINITY))
			})
			.min_by(|a, b| a.partial_cmp(b).unwrap())
			.unwrap()
	});
	let mut y_max = options.y_max.unwrap_or_else(|| {
		series
			.iter()
			.flat_map(|series| {
				series
					.data
					.iter()
					.map(|p| p.y.as_ref().map(|y| y.max).unwrap_or(f64::NEG_INFINITY))
			})
			.max_by(|a, b| a.partial_cmp(b).unwrap())
			.unwrap()
	});
	if !y_max.is_finite() || (y_max - y_min).abs() < f64::EPSILON {
		y_max = y_min + 1.0;
	}

	// Compute the boxes.
	let ComputeBoxesOutput {
		chart_box,
		x_axis_labels_box,
		x_axis_title_box,
		y_axis_grid_line_info,
		y_axis_labels_box,
		y_axis_title_box,
	} = compute_boxes(ComputeBoxesOptions {
		chart_config,
		ctx: ctx.clone(),
		height,
		include_x_axis_labels: options.should_draw_x_axis_labels.unwrap_or(true),
		include_x_axis_title: x_axis_title.is_some(),
		include_y_axis_labels: options.should_draw_y_axis_labels.unwrap_or(true),
		include_y_axis_title: y_axis_title.is_some(),
		width,
		x_axis_grid_line_interval: None,
		y_axis_grid_line_interval: None,
		y_max,
		y_min,
	});

	let categories: Vec<&String> = series[0].data.iter().map(|point| &point.label).collect();
	let box_group_width = (chart_box.w
		- chart_config.bar_group_gap * (categories.len() + 1).to_f64().unwrap())
		/ categories.len().to_f64().unwrap();

	// Draw the X axis labels.
	if options.should_draw_x_axis_labels.unwrap_or(true) {
		draw_bar_chart_x_axis_labels(DrawBarChartXAxisLabelsOptions {
			bar_group_gap: chart_config.bar_group_gap,
			chart_colors,
			rect: x_axis_labels_box,
			categories: &categories,
			ctx: ctx.clone(),
			group_width: box_group_width,
			width,
		})
	}

	draw_y_axis_grid_lines(DrawYAxisGridLinesOptions {
		chart_colors,
		chart_config,
		ctx: ctx.clone(),
		rect: chart_box,
		y_axis_grid_line_info: y_axis_grid_line_info.clone(),
	});

	draw_x_axis(DrawXAxisOptions {
		chart_colors,
		chart_config,
		ctx: ctx.clone(),
		rect: chart_box,
		y_axis_grid_line_info: y_axis_grid_line_info.clone(),
	});

	// Draw the Y axis labels.
	if options.should_draw_y_axis_labels.unwrap_or(true) {
		draw_y_axis_labels(DrawYAxisLabelsOptions {
			rect: y_axis_labels_box,
			ctx: ctx.clone(),
			font_size: chart_config.font_size,
			grid_line_info: y_axis_grid_line_info,
			height,
		})
	}

	// Draw the X axis title.
	if let Some(x_axis_title) = x_axis_title {
		draw_x_axis_title(DrawXAxisTitleOptions {
			rect: x_axis_title_box,
			ctx: ctx.clone(),
			title: x_axis_title,
		});
	}

	// Draw the Y axis title.
	if let Some(y_axis_title) = y_axis_title {
		draw_y_axis_title(DrawYAxisTitleOptions {
			rect: y_axis_title_box,
			ctx: ctx.clone(),
			title: y_axis_title,
		});
	}

	// Draw the boxes.
	for (series_index, this_series) in series.iter().enumerate() {
		for (point_index, point) in this_series.data.iter().enumerate() {
			let output = draw_box(DrawBoxOptions {
				box_gap: chart_config.bar_gap,
				box_group_gap: chart_config.bar_group_gap,
				box_group_width,
				chart_box,
				chart_config,
				point_index,
				series_index,
				y_max,
				y_min,
				ctx: ctx.clone(),
				data: series,
				point: point.clone(),
				series: this_series,
			});
			hover_regions.extend(output.hover_regions);
		}
	}

	let overlay_info = BoxChartOverlayInfo { chart_box };

	DrawChartOutput {
		hover_regions,
		overlay_info,
	}
}

fn draw_box_chart_overlay(
	options: DrawOverlayOptions<BoxChartOverlayInfo, BoxChartHoverRegionInfo>,
) {
	todo!()
	// 	let {
	// 		activeHoverRegions,
	// 		ctx,
	// 		info: { chartBox },
	// 		overlayDiv,
	// 	} = options
	// 	let tooltips: TooltipLabel[] = []
	// 	let boxPointIndexForName: { [key: string]: number } = {
	// 		max: 4,
	// 		median: 2,
	// 		min: 0,
	// 		p25: 1,
	// 		p75: 3,
	// 	}
	// 	activeHoverRegions.sort((activeHoverRegionA, activeHoverRegionB) => {
	// 		let boxPointIndexA = boxPointIndexForName[activeHoverRegionA.info.name]
	// 		if (boxPointIndexA === undefined) throw Error()
	// 		let boxPointIndexB = boxPointIndexForName[activeHoverRegionB.info.name]
	// 		if (boxPointIndexB === undefined) throw Error()
	// 		return boxPointIndexA > boxPointIndexB ? -1 : 1
	// 	})
	// 	for (let i = 0; i < activeHoverRegions.length; i++) {
	// 		let activeHoverRegion = activeHoverRegions[i]
	// 		if (activeHoverRegion === undefined) throw Error()
	// 		let color = activeHoverRegion.info.color
	// 		let x = activeHoverRegion.info.label
	// 		let name = activeHoverRegion.info.name
	// 		let value = formatNumber(activeHoverRegion.info.value)
	// 		let y = `${name} = ${value}`
	// 		let text = `(${x}, ${y})`
	// 		tooltips.push({
	// 			color,
	// 			text,
	// 		})
	// 	}
	// 	for (let activeHoverRegion of activeHoverRegions) {
	// 		drawLine({
	// 			color: chartColors.current.crosshairsColor,
	// 			ctx,
	// 			dashed: true,
	// 			end: {
	// 				x: chartBox.x + chartBox.w,
	// 				y: activeHoverRegion.info.tooltipOriginPixels.y,
	// 			},
	// 			start: { x: chartBox.x, y: activeHoverRegion.info.tooltipOriginPixels.y },
	// 		})
	// 	}
	// 	if (tooltips.length > 0) {
	// 		let lastActiveHoverRegion =
	// 			activeHoverRegions[activeHoverRegions.length - 1]
	// 		if (lastActiveHoverRegion === undefined) throw Error()
	// 		let origin = lastActiveHoverRegion.info.tooltipOriginPixels
	// 		drawTooltip({
	// 			centerHorizontal: true,
	// 			container: overlayDiv,
	// 			labels: tooltips,
	// 			origin,
	// 		})
	// 	}
}

struct DrawBoxOptions<'a> {
	box_gap: f64,
	box_group_gap: f64,
	box_group_width: f64,
	chart_box: Rect,
	chart_config: &'a ChartConfig,
	ctx: CanvasRenderingContext2d,
	data: &'a [BoxChartSeries],
	point: BoxChartPoint,
	point_index: usize,
	series: &'a BoxChartSeries,
	series_index: usize,
	y_max: f64,
	y_min: f64,
}

struct DrawBoxOutput {
	hover_regions: Vec<HoverRegion<BoxChartHoverRegionInfo>>,
}

fn draw_box(options: DrawBoxOptions) -> DrawBoxOutput {
	let DrawBoxOptions {
		chart_config,
		box_gap,
		box_group_gap,
		box_group_width,
		chart_box,
		ctx,
		data,
		point,
		point_index,
		series,
		series_index,
		y_max,
		y_min,
	} = options;
	let n_series = data.len().to_f64().unwrap();
	let mut hover_regions: Vec<HoverRegion<BoxChartHoverRegionInfo>> = Vec::new();

	let value = if let Some(y) = point.y {
		y
	} else {
		return DrawBoxOutput { hover_regions };
	};

	let box_width = box_group_width / n_series - chart_config.bar_gap * (n_series - 1.0);
	let x = chart_box.x
		+ (box_group_gap + (box_group_gap + box_group_width) * point_index.to_f64().unwrap())
		+ (box_gap + box_width) * series_index.to_f64().unwrap();

	let whisker_tip_width = box_width / 10.0;
	let line_width = 2.0;
	let value_to_pixels = |value: f64| {
		chart_box.y + chart_box.h
			- (-y_min / (y_max - y_min)) * chart_box.h
			- (value / (y_max - y_min)) * chart_box.h
	};

	// Draw the box.
	let rect = Rect {
		h: ((value.p75 - value.p25).abs() / (y_max - y_min)) * chart_box.h,
		w: box_width,
		x,
		y: value_to_pixels(f64::max(value.p25, value.p75)),
	};
	let radius = f64::INFINITY
		.min((rect.h / 2.0).abs())
		.min((rect.w / 6.0).abs())
		.min(chart_config.max_corner_radius);
	draw_rounded_rect(DrawRoundedRectOptions {
		rect,
		ctx: ctx.clone(),
		fill_color: Some(&format!("{}af", series.color)),
		radius,
		stroke_color: Some(&series.color),
		stroke_width: Some(chart_config.bar_stroke_width),
		round_bottom_left: true,
		round_bottom_right: true,
		round_top_left: true,
		round_top_right: true,
	});

	// Create a clip path so the median line will not overflow the box.
	ctx.save();
	draw_rounded_rect(DrawRoundedRectOptions {
		ctx: ctx.clone(),
		fill_color: None,
		radius,
		rect,
		round_bottom_left: true,
		round_bottom_right: true,
		round_top_left: true,
		round_top_right: true,
		stroke_color: None,
		stroke_width: None,
	});
	ctx.clip();
	// Draw the median line.
	let median_box = Rect {
		h: line_width,
		w: box_width,
		x,
		y: value_to_pixels(value.p50),
	};
	draw_line(DrawLineOptions {
		color: Some(&series.color),
		ctx: ctx.clone(),
		end: Point {
			x: median_box.x + median_box.w,
			y: median_box.y,
		},
		line_width: Some(line_width),
		start: Point {
			x: median_box.x,
			y: median_box.y,
		},
		dashed: None,
		line_cap: None,
	});
	hover_regions.push(box_chart_hover_region(BoxChartHoverRegionOptions {
		rect: median_box,
		color: series.color.clone(),
		label: point.label.clone(),
		name: "median".to_owned(),
		tooltip_origin_pixels: Point {
			x: x + box_width / 2.0,
			y: median_box.y,
		},
		value: value.p50,
		chart_config,
	}));
	ctx.restore();

	// Draw the min line.
	draw_line(DrawLineOptions {
		color: Some(&series.color),
		ctx: ctx.clone(),
		end: Point {
			x: x + box_width / 2.0,
			y: value_to_pixels(value.min),
		},
		line_width: Some(line_width),
		start: Point {
			x: x + box_width / 2.0,
			y: value_to_pixels(value.p25),
		},
		dashed: None,
		line_cap: None,
	});
	let min_whisker_tip_box = Rect {
		h: line_width,
		w: whisker_tip_width,
		x: x + box_width / 2.0 - whisker_tip_width / 2.0,
		y: value_to_pixels(value.min),
	};
	draw_line(DrawLineOptions {
		color: Some(&series.color),
		ctx: ctx.clone(),
		end: Point {
			x: min_whisker_tip_box.x + min_whisker_tip_box.w,
			y: min_whisker_tip_box.y,
		},
		line_cap: Some("round"),
		line_width: Some(line_width),
		start: Point {
			x: min_whisker_tip_box.x,
			y: min_whisker_tip_box.y,
		},
		dashed: None,
	});
	hover_regions.push(box_chart_hover_region(BoxChartHoverRegionOptions {
		rect: min_whisker_tip_box,
		color: series.color.clone(),
		label: point.label.clone(),
		name: "min".to_owned(),
		tooltip_origin_pixels: Point {
			y: min_whisker_tip_box.y,
			x: x + box_width / 2.0,
		},
		value: value.min,
		chart_config,
	}));

	// Draw the max line.
	draw_line(DrawLineOptions {
		color: Some(&series.color),
		ctx: ctx.clone(),
		end: Point {
			x: x + box_width / 2.0,
			y: value_to_pixels(value.max),
		},
		line_width: Some(line_width),
		start: Point {
			x: x + box_width / 2.0,
			y: value_to_pixels(value.p75),
		},
		dashed: None,
		line_cap: None,
	});
	let max_whisker_tip_box = Rect {
		h: line_width,
		w: whisker_tip_width,
		x: x + box_width / 2.0 - whisker_tip_width / 2.0,
		y: value_to_pixels(value.max),
	};
	draw_line(DrawLineOptions {
		color: Some(&series.color),
		ctx,
		end: Point {
			x: max_whisker_tip_box.x + max_whisker_tip_box.w,
			y: max_whisker_tip_box.y,
		},
		line_cap: Some("round"),
		line_width: Some(line_width),
		start: Point {
			x: max_whisker_tip_box.x,
			y: max_whisker_tip_box.y,
		},
		dashed: None,
	});
	hover_regions.push(box_chart_hover_region(BoxChartHoverRegionOptions {
		rect: max_whisker_tip_box,
		color: series.color.clone(),
		label: point.label.clone(),
		name: "max".to_owned(),
		tooltip_origin_pixels: Point {
			x: x + box_width / 2.0,
			y: max_whisker_tip_box.y,
		},
		value: value.max,
		chart_config,
	}));

	// Register the p25 hit region.
	let p25_box = Rect {
		h: line_width,
		w: box_width,
		x,
		y: value_to_pixels(value.p25),
	};
	hover_regions.push(box_chart_hover_region(BoxChartHoverRegionOptions {
		rect: p25_box,
		color: series.color.clone(),
		label: point.label.clone(),
		name: "p25".to_owned(),
		tooltip_origin_pixels: Point {
			x: x + box_width / 2.0,
			y: p25_box.y,
		},
		value: value.p25,
		chart_config,
	}));

	// Register the p75 hit region.
	let p75_box = Rect {
		h: line_width,
		w: box_width,
		x,
		y: value_to_pixels(value.p75),
	};
	hover_regions.push(box_chart_hover_region(BoxChartHoverRegionOptions {
		rect: p75_box,
		color: series.color.clone(),
		label: point.label,
		name: "p75".to_owned(),
		tooltip_origin_pixels: Point {
			x: x + box_width / 2.0,
			y: p75_box.y,
		},
		value: value.p75,
		chart_config,
	}));

	DrawBoxOutput { hover_regions }
}

struct BoxChartHoverRegionOptions<'a> {
	chart_config: &'a ChartConfig,
	rect: Rect,
	color: String,
	label: String,
	name: String,
	tooltip_origin_pixels: Point,
	value: f64,
}

fn box_chart_hover_region(
	options: BoxChartHoverRegionOptions,
) -> HoverRegion<BoxChartHoverRegionInfo> {
	let BoxChartHoverRegionOptions {
		chart_config,
		rect,
		color,
		label,
		name,
		tooltip_origin_pixels,
		value,
	} = options;
	let tooltip_target_radius = chart_config.tooltip_target_radius;
	HoverRegion {
		distance: Box::new(move |x, y| (rect.x - x).powi(2) + (rect.y - y).powi(2)),
		hit_test: Box::new(move |x, y| {
			y < rect.y + rect.h + tooltip_target_radius
				&& y > rect.y - rect.h - tooltip_target_radius
				&& x > rect.x - tooltip_target_radius
				&& x < rect.x + rect.w + tooltip_target_radius
		}),
		info: BoxChartHoverRegionInfo {
			color,
			label,
			name,
			tooltip_origin_pixels,
			value,
		},
	}
}

struct DrawLineOptions<'a> {
	color: Option<&'a str>,
	ctx: CanvasRenderingContext2d,
	dashed: Option<bool>,
	end: Point,
	line_cap: Option<&'a str>,
	line_width: Option<f64>,
	start: Point,
}

fn draw_line(options: DrawLineOptions) {
	let DrawLineOptions {
		color,
		ctx,
		dashed,
		end,
		line_cap,
		line_width,
		start,
	} = options;
	let line_width = line_width.unwrap_or(1.0);
	let dashed = dashed.unwrap_or(false);
	let line_cap = line_cap.as_deref().unwrap_or("butt");
	ctx.save();
	if dashed {
		ctx.set_line_dash(&JsValue::from_serde(&[4.0, 4.0]).unwrap())
			.unwrap();
	}
	if let Some(color) = &color {
		ctx.set_stroke_style(&color.to_owned().into());
	}
	ctx.set_line_width(line_width);
	ctx.set_line_cap(line_cap);
	ctx.begin_path();
	ctx.move_to(start.x, start.y);
	ctx.line_to(end.x, end.y);
	ctx.stroke();
	ctx.restore();
}
