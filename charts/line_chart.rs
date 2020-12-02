use crate::{
	chart::{
		ActiveHoverRegion, ChartImpl, DrawChartOptions, DrawChartOutput, DrawOverlayOptions,
		HoverRegion,
	},
	common::{
		compute_boxes, compute_x_axis_grid_line_info, draw_x_axis, draw_x_axis_grid_lines,
		draw_x_axis_labels, draw_x_axis_title, draw_y_axis, draw_y_axis_grid_lines,
		draw_y_axis_labels, draw_y_axis_title, ComputeBoxesOptions, ComputeBoxesOutput,
		ComputeXAxisGridLineInfoOptions, DrawXAxisGridLinesOptions, DrawXAxisLabelsOptions,
		DrawXAxisOptions, DrawXAxisTitleOptions, DrawYAxisGridLinesOptions, DrawYAxisLabelsOptions,
		DrawYAxisOptions, DrawYAxisTitleOptions, GridLineInterval, Point, Rect,
	},
	config::ChartConfig,
};
use itertools::Itertools;
use num_traits::ToPrimitive;
use wasm_bindgen::JsValue;
use web_sys::*;

pub struct LineChart;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LineChartOptions {
	pub hide_legend: Option<bool>,
	pub labels: Option<Vec<String>>,
	pub series: Vec<LineChartSeries>,
	pub should_draw_x_axis_labels: Option<bool>,
	pub should_draw_y_axis_labels: Option<bool>,
	pub title: Option<String>,
	pub x_axis_grid_line_interval: Option<GridLineInterval>,
	pub x_axis_title: Option<String>,
	pub x_max: Option<f64>,
	pub x_min: Option<f64>,
	pub y_axis_grid_line_interval: Option<GridLineInterval>,
	pub y_axis_title: Option<String>,
	pub y_max: Option<f64>,
	pub y_min: Option<f64>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LineChartSeries {
	pub color: String,
	pub data: Vec<LineChartPoint>,
	pub line_style: Option<LineStyle>,
	pub point_style: Option<PointStyle>,
	pub title: Option<String>,
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct LineChartPoint {
	pub x: f64,
	pub y: f64,
}

impl Into<Point> for LineChartPoint {
	fn into(self) -> Point {
		Point {
			x: self.x,
			y: self.y,
		}
	}
}

impl From<Point> for LineChartPoint {
	fn from(value: Point) -> LineChartPoint {
		LineChartPoint {
			x: value.x,
			y: value.y,
		}
	}
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum LineStyle {
	#[serde(rename = "hidden")]
	Hidden,
	#[serde(rename = "solid")]
	Solid,
	#[serde(rename = "dashed")]
	Dashed,
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum PointStyle {
	#[serde(rename = "hidden")]
	Hidden,
	#[serde(rename = "circle")]
	Circle,
}

pub struct LineChartOverlayInfo {
	chart_box: Rect,
	x_max: f64,
	x_min: f64,
	y_max: f64,
	y_min: f64,
}

#[derive(Clone)]
pub struct LineChartHoverRegionInfo {
	chart_box: Rect,
	color: String,
	point: Point,
	point_label: Option<String>,
	point_value: f64,
	series_index: f64,
	series_title: Option<String>,
	tooltip_origin_pixels: Point,
	x_max: f64,
	x_min: f64,
	y_max: f64,
	y_min: f64,
}

impl ChartImpl for LineChart {
	type Options = LineChartOptions;
	type OverlayInfo = LineChartOverlayInfo;
	type HoverRegionInfo = LineChartHoverRegionInfo;

	fn draw_chart(
		options: DrawChartOptions<Self::Options>,
	) -> DrawChartOutput<Self::OverlayInfo, Self::HoverRegionInfo> {
		draw_line_chart(options)
	}

	fn draw_overlay(options: DrawOverlayOptions<Self::OverlayInfo, Self::HoverRegionInfo>) {
		draw_line_chart_overlay(options)
	}
}

fn draw_line_chart(
	options: DrawChartOptions<LineChartOptions>,
) -> DrawChartOutput<LineChartOverlayInfo, LineChartHoverRegionInfo> {
	let DrawChartOptions {
		chart_colors,
		chart_config,
		options,
		ctx,
		..
	} = options;
	let LineChartOptions {
		labels,
		series,
		x_axis_grid_line_interval,
		x_axis_title,
		y_axis_grid_line_interval,
		y_axis_title,
		..
	} = &options;
	let canvas = ctx.canvas().unwrap();
	let width = canvas.client_width().to_f64().unwrap();
	let height = canvas.client_height().to_f64().unwrap();
	let mut hover_regions: Vec<HoverRegion<LineChartHoverRegionInfo>> = Vec::new();

	// Compute the bounds.
	let x_min = options.x_min.unwrap_or_else(|| {
		options
			.series
			.iter()
			.flat_map(|series| series.data.iter().map(|point| point.x))
			.min_by(|a, b| a.partial_cmp(b).unwrap())
			.unwrap()
	});
	let x_max = options.x_max.unwrap_or_else(|| {
		options
			.series
			.iter()
			.flat_map(|series| series.data.iter().map(|point| point.x))
			.max_by(|a, b| a.partial_cmp(b).unwrap())
			.unwrap()
	});
	let y_min = options.y_min.unwrap_or_else(|| {
		options
			.series
			.iter()
			.flat_map(|series| series.data.iter().map(|point| point.y))
			.min_by(|a, b| a.partial_cmp(b).unwrap())
			.unwrap()
	});
	let y_max = options.y_max.unwrap_or_else(|| {
		options
			.series
			.iter()
			.flat_map(|series| series.data.iter().map(|point| point.y))
			.max_by(|a, b| a.partial_cmp(b).unwrap())
			.unwrap()
	});

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
		x_axis_grid_line_interval: x_axis_grid_line_interval.clone(),
		y_axis_grid_line_interval: y_axis_grid_line_interval.clone(),
		y_max,
		y_min,
	});

	// Compute the grid line info.
	let x_axis_grid_line_info = compute_x_axis_grid_line_info(ComputeXAxisGridLineInfoOptions {
		chart_width: chart_box.w,
		ctx: ctx.clone(),
		x_axis_grid_line_interval: x_axis_grid_line_interval.clone(),
		x_max,
		x_min,
	});

	draw_x_axis_grid_lines(DrawXAxisGridLinesOptions {
		chart_colors,
		chart_config,
		ctx: ctx.clone(),
		rect: chart_box,
		x_axis_grid_line_info: x_axis_grid_line_info.clone(),
	});

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

	draw_y_axis(DrawYAxisOptions {
		chart_colors,
		chart_config,
		ctx: ctx.clone(),
		rect: chart_box,
		x_axis_grid_line_info: x_axis_grid_line_info.clone(),
	});

	// Draw the X axis labels.
	if options.should_draw_x_axis_labels.unwrap_or(true) {
		draw_x_axis_labels(DrawXAxisLabelsOptions {
			rect: x_axis_labels_box,
			ctx: ctx.clone(),
			grid_line_info: x_axis_grid_line_info,
			width,
			labels,
		})
	}

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

	// Draw the lines.
	for series in series.iter() {
		draw_line(DrawLineOptions {
			chart_config,
			chart_box,
			ctx: ctx.clone(),
			series,
			x_max,
			x_min,
			y_max,
			y_min,
		});
	}

	let max_point_count = series
		.iter()
		.map(|series| series.data.len())
		.max()
		.unwrap()
		.to_f64()
		.unwrap();
	let should_draw_points = chart_box.w / max_point_count > 2.0 * chart_config.point_radius;

	// Draw the points.
	if should_draw_points {
		for series in series.iter() {
			for point in series.data.iter() {
				draw_point(DrawPointOptions {
					chart_box,
					color: &series.color,
					ctx: ctx.clone(),
					point: *point,
					point_style: series.point_style.unwrap_or(PointStyle::Circle),
					radius: chart_config.point_radius,
					x_max,
					x_min,
					y_max,
					y_min,
				})
			}
		}
	}

	// Compute the hover regions.
	let has_multiple_series = series.len() > 1;
	for (series_index, series) in series.iter().enumerate() {
		for (point_index, point) in series.data.iter().enumerate() {
			let point_pixels = point_to_pixels(PointToPixelsOptions {
				chart_box,
				point: point.clone().into(),
				x_max,
				x_min,
				y_max,
				y_min,
			});
			let tooltip_target_radius = chart_config.tooltip_target_radius;
			let point_label = labels
				.as_ref()
				.map(|labels| labels.get(point_index).unwrap().clone());
			let hover_region = HoverRegion {
				distance: Box::new(move |x: f64, y: f64| {
					(point_pixels.x - x).powi(2) + (point_pixels.y - y).powi(2)
				}),
				hit_test: Box::new(move |x: f64, y: f64| {
					x > point_pixels.x - tooltip_target_radius
						&& x < point_pixels.x + tooltip_target_radius
						&& y > point_pixels.y - tooltip_target_radius
						&& y < point_pixels.y + tooltip_target_radius
				}),
				info: LineChartHoverRegionInfo {
					chart_box,
					color: series.color.clone(),
					point: point.clone().into(),
					point_label,
					point_value: point.y,
					series_index: series_index.to_f64().unwrap(),
					series_title: if has_multiple_series {
						series.title.clone()
					} else {
						None
					},
					tooltip_origin_pixels: point_pixels,
					x_max,
					x_min,
					y_max,
					y_min,
				},
			};
			hover_regions.push(hover_region);
		}
	}

	let overlay_info = LineChartOverlayInfo {
		chart_box,
		x_max,
		x_min,
		y_max,
		y_min,
	};

	DrawChartOutput {
		hover_regions,
		overlay_info,
	}
}

struct DrawPointOptions<'a> {
	chart_box: Rect,
	color: &'a str,
	ctx: CanvasRenderingContext2d,
	point: LineChartPoint,
	point_style: PointStyle,
	radius: f64,
	x_max: f64,
	x_min: f64,
	y_max: f64,
	y_min: f64,
}

fn draw_point(options: DrawPointOptions) {
	let DrawPointOptions {
		chart_box,
		color,
		ctx,
		point,
		point_style,
		radius,
		x_max,
		x_min,
		y_max,
		y_min,
	} = options;
	if let PointStyle::Hidden = point_style {
		return;
	}
	let point_pixels = point_to_pixels(PointToPixelsOptions {
		chart_box,
		point: point.clone().into(),
		x_max,
		x_min,
		y_max,
		y_min,
	});
	ctx.begin_path();
	ctx.set_fill_style(&color.into());
	ctx.arc(
		point_pixels.x,
		point_pixels.y,
		radius,
		0.0,
		2.0 * std::f64::consts::PI,
	)
	.unwrap();
	ctx.fill();
}

struct DrawLineOptions<'a> {
	chart_box: Rect,
	chart_config: &'a ChartConfig,
	ctx: CanvasRenderingContext2d,
	series: &'a LineChartSeries,
	x_max: f64,
	x_min: f64,
	y_max: f64,
	y_min: f64,
}

fn draw_line(options: DrawLineOptions) {
	let DrawLineOptions {
		chart_box,
		chart_config,
		ctx,
		series,
		x_max,
		x_min,
		y_max,
		y_min,
	} = options;
	if let Some(LineStyle::Hidden) = &series.line_style {
		return;
	}
	ctx.save();
	ctx.begin_path();
	ctx.set_stroke_style(&series.color.as_str().into());
	if let Some(LineStyle::Dashed) = &series.line_style {
		ctx.set_line_dash(&JsValue::from_serde(&[4, 4]).unwrap())
			.unwrap();
	}
	if series.data.len() < 2 {
		return;
	}
	let first_point = series.data[0];
	let first_point_pixels = point_to_pixels(PointToPixelsOptions {
		chart_box,
		point: first_point.into(),
		x_max,
		x_min,
		y_max,
		y_min,
	});
	ctx.move_to(first_point_pixels.x, first_point_pixels.y);
	let mut cp1 = first_point;
	for (previous_point, point, next_point) in series.data.iter().tuple_windows() {
		let (cp2, next_cp1) = interpolate_spline(InterpolateSplineOptions {
			next_point: next_point.clone().into(),
			point: point.clone().into(),
			previous_point: previous_point.clone().into(),
			tension: chart_config.spline_tension,
		});
		let cp1_pixels = point_to_pixels(PointToPixelsOptions {
			chart_box,
			point: cp1.into(),
			x_max,
			x_min,
			y_max,
			y_min,
		});
		let cp2_pixels = point_to_pixels(PointToPixelsOptions {
			chart_box,
			point: cp2,
			x_max,
			x_min,
			y_max,
			y_min,
		});
		let point_pixels = point_to_pixels(PointToPixelsOptions {
			chart_box,
			point: point.clone().into(),
			x_max,
			x_min,
			y_max,
			y_min,
		});
		ctx.bezier_curve_to(
			cp1_pixels.x,
			cp1_pixels.y,
			cp2_pixels.x,
			cp2_pixels.y,
			point_pixels.x,
			point_pixels.y,
		);
		cp1 = next_cp1.clone().into();
	}
	let last_point = series.data[series.data.len() - 1];
	let last_point_pixels = point_to_pixels(PointToPixelsOptions {
		chart_box,
		point: last_point.into(),
		x_max,
		x_min,
		y_max,
		y_min,
	});
	let cp1_pixels = point_to_pixels(PointToPixelsOptions {
		chart_box,
		point: cp1.into(),
		x_max,
		x_min,
		y_max,
		y_min,
	});
	ctx.bezier_curve_to(
		cp1_pixels.x,
		cp1_pixels.y,
		last_point_pixels.x,
		last_point_pixels.y,
		last_point_pixels.x,
		last_point_pixels.y,
	);
	ctx.stroke();
	ctx.restore();
}

struct InterpolateSplineOptions {
	next_point: Point,
	point: Point,
	previous_point: Point,
	tension: f64,
}

fn interpolate_spline(options: InterpolateSplineOptions) -> (Point, Point) {
	let InterpolateSplineOptions {
		next_point,
		point,
		previous_point,
		tension,
	} = options;
	let d01 = ((point.x - previous_point.x).powi(2) + (point.y - previous_point.y).powi(2)).sqrt();
	let d12 = ((point.x - next_point.x).powi(2) + (point.y - next_point.y).powi(2)).sqrt();
	let m01 = (tension * d01) / (d01 + d12);
	let m12 = (tension * d12) / (d01 + d12);
	let cp1 = Point {
		x: point.x - m01 * (next_point.x - previous_point.x),
		y: point.y - m01 * (next_point.y - previous_point.y),
	};
	let cp2 = Point {
		x: point.x + m12 * (next_point.x - previous_point.x),
		y: point.y + m12 * (next_point.y - previous_point.y),
	};
	(cp1, cp2)
}

fn draw_line_chart_overlay(
	options: DrawOverlayOptions<LineChartOverlayInfo, LineChartHoverRegionInfo>,
) {
	let DrawOverlayOptions {
		active_hover_regions,
		ctx,
		overlay_info,
		overlay_div,
		..
	} = options;
	let LineChartOverlayInfo {
		chart_box,
		x_max,
		x_min,
		y_max,
		y_min,
	} = &overlay_info;
	// 	let closestActiveHoverRegionForSeries = new Map<
	// 		number,
	// 		ActiveHoverRegion<LineChartHoverRegionInfo>
	// 	>()
	// 	for (let activeHoverRegion of activeHoverRegions) {
	// 		let activeHoverRegionForSeries = closestActiveHoverRegionForSeries.get(
	// 			activeHoverRegion.info.seriesIndex,
	// 		)
	// 		if (
	// 			!activeHoverRegionForSeries ||
	// 			activeHoverRegion.distance < activeHoverRegionForSeries.distance
	// 		) {
	// 			closestActiveHoverRegionForSeries.set(
	// 				activeHoverRegion.info.seriesIndex,
	// 				activeHoverRegion,
	// 			)
	// 		}
	// 	}
	// 	let closestActiveHoverRegions = Array.from(
	// 		closestActiveHoverRegionForSeries.values(),
	// 	)
	// 	let tooltips: TooltipLabel[] = closestActiveHoverRegions.map(
	// 		activeHoverRegion => {
	// 			let pointLabel = activeHoverRegion.info.pointLabel
	// 			if (pointLabel == undefined) {
	// 				pointLabel = formatNumber(activeHoverRegion.info.point.x)
	// 			}
	// 			let pointValue = formatNumber(activeHoverRegion.info.point.y)
	// 			let seriesTitle = activeHoverRegion.info.seriesTitle
	// 			let text
	// 			if (seriesTitle === undefined) {
	// 				text = `(${pointLabel}, ${pointValue})`
	// 			} else {
	// 				text = `${seriesTitle} (${pointLabel}, ${pointValue})`
	// 			}
	// 			return {
	// 				color: activeHoverRegion.info.color,
	// 				text,
	// 			}
	// 		},
	// 	)
	// 	let closestActiveHoverRegion:
	// 		| ActiveHoverRegion<LineChartHoverRegionInfo>
	// 		| undefined
	// 	for (let activeHoverRegion of closestActiveHoverRegions) {
	// 		if (
	// 			!closestActiveHoverRegion ||
	// 			activeHoverRegion.distance < closestActiveHoverRegion.distance
	// 		) {
	// 			closestActiveHoverRegion = activeHoverRegion
	// 		}
	// 	}
	// 	let tooltipOrigin = closestActiveHoverRegion
	// 		? closestActiveHoverRegion.info.tooltipOriginPixels
	// 		: undefined
	// 	if (tooltipOrigin && tooltips.length === 1) {
	// 		drawCrosshairs({
	// 			chart_box,
	// 			crosshairsColor: chartColors.current.crosshairsColor,
	// 			ctx,
	// 			origin: tooltipOrigin,
	// 		})
	// 	}
	// 	if (tooltipOrigin) {
	// 		drawTooltip({
	// 			container: overlayDiv,
	// 			labels: tooltips,
	// 			origin: tooltipOrigin,
	// 		})
	// 	}
	// 	closestActiveHoverRegions.forEach(activeHoverRegion => {
	// 		let point = activeHoverRegion.info.point
	// 		drawPoint({
	// 			chart_box,
	// 			color: activeHoverRegion.info.color,
	// 			ctx,
	// 			point: { x: point.x, y: point.y },
	// 			pointStyle: PointStyle.Circle,
	// 			radius: chartConfig.pointRadius,
	// 			xMax,
	// 			xMin,
	// 			yMax,
	// 			yMin,
	// 		})
	// 		drawPoint({
	// 			chart_box,
	// 			color: "#00000022",
	// 			ctx,
	// 			point: { x: point.x, y: point.y },
	// 			pointStyle: PointStyle.Circle,
	// 			radius: chartConfig.pointRadius,
	// 			xMax,
	// 			xMin,
	// 			yMax,
	// 			yMin,
	// 		})
	// 	})
}

struct DrawCrosshairsOptions {
	chart_box: Rect,
	crosshairs_color: String,
	ctx: CanvasRenderingContext2d,
	origin: Point,
}

fn draw_crosshairs(options: DrawCrosshairsOptions) {
	let DrawCrosshairsOptions {
		chart_box,
		crosshairs_color,
		ctx,
		origin,
	} = options;
	ctx.save();
	ctx.begin_path();
	ctx.set_line_dash(&JsValue::from_serde(&[4, 4]).unwrap())
		.unwrap();
	ctx.set_stroke_style(&crosshairs_color.into());
	ctx.move_to(origin.x, chart_box.y);
	ctx.line_to(origin.x, chart_box.y + chart_box.h);
	ctx.move_to(chart_box.x, origin.y);
	ctx.line_to(chart_box.x + chart_box.w, origin.y);
	ctx.stroke();
	ctx.restore();
}

struct PointToPixelsOptions {
	chart_box: Rect,
	point: Point,
	x_max: f64,
	x_min: f64,
	y_max: f64,
	y_min: f64,
}

fn point_to_pixels(options: PointToPixelsOptions) -> Point {
	let PointToPixelsOptions {
		chart_box,
		point,
		x_max,
		x_min,
		y_max,
		y_min,
	} = options;
	Point {
		x: chart_box.x
			+ (-x_min / (x_max - x_min)) * chart_box.w
			+ (point.x / (x_max - x_min)) * chart_box.w,
		y: chart_box.y + chart_box.h
			- (-y_min / (y_max - y_min)) * chart_box.h
			- (point.y / (y_max - y_min)) * chart_box.h,
	}
}
