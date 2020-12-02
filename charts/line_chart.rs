use crate::{
	chart::{ActiveHoverRegion, DrawChartOptions, HoverRegion},
	common::GridLineInterval,
	common::{Point, Rect},
};
use wasm_bindgen::JsValue;
use web_sys::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LineChartOptions {
	pub hide_legend: Option<bool>,
	pub labels: Option<String>,
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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LineChartPoint {
	pub x: f64,
	pub y: Option<f64>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum LineStyle {
	#[serde(rename = "hidden")]
	Hidden,
	#[serde(rename = "solid")]
	Solid,
	#[serde(rename = "dashed")]
	Dashed,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum PointStyle {
	#[serde(rename = "hidden")]
	Hidden,
	#[serde(rename = "circle")]
	Circle,
}

struct LineChartOverlayInfo {
	chart_box: Rect,
	x_max: f64,
	x_min: f64,
	y_max: f64,
	y_min: f64,
}

#[derive(Clone)]
struct LineChartHoverRegionInfo {
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

struct DrawLineChartOutput {
	hover_regions: Vec<HoverRegion<LineChartHoverRegionInfo>>,
	overlay_info: LineChartOverlayInfo,
}

fn draw_line_chart(options: DrawChartOptions<LineChartOptions>) -> DrawLineChartOutput {
	todo!()
	// 	let {
	// 		labels,
	// 		series: data,
	// 		xAxisGridLineInterval,
	// 		xAxisTitle,
	// 		yAxisGridLineInterval,
	// 		yAxisTitle,
	// 	} = options
	// 	let width = ctx.canvas.clientWidth
	// 	let height = ctx.canvas.clientHeight
	// 	let hoverRegions: Array<HoverRegion<LineChartHoverRegionInfo>> = []

	// 	// Compute the bounds.
	// 	let xMin: number
	// 	if (options.xMin !== undefined) {
	// 		xMin = options.xMin
	// 	} else {
	// 		xMin = Math.min(
	// 			...options.series.flatMap(series => series.data.map(({ x }) => x)),
	// 		)
	// 	}
	// 	let xMax: number
	// 	if (options.xMax !== undefined) {
	// 		xMax = options.xMax
	// 	} else {
	// 		xMax = Math.max(
	// 			...options.series.flatMap(series => series.data.map(({ x }) => x)),
	// 		)
	// 	}
	// 	let yMin: number
	// 	if (options.yMin !== undefined) {
	// 		yMin = options.yMin
	// 	} else {
	// 		yMin = Math.min(
	// 			...options.series.flatMap(series =>
	// 				series.data.map(p => p.y ?? Infinity),
	// 			),
	// 		)
	// 	}
	// 	let yMax: number
	// 	if (options.yMax !== undefined) {
	// 		yMax = options.yMax
	// 	} else {
	// 		yMax = Math.max(
	// 			...options.series.flatMap(series =>
	// 				series.data.map(p => p.y ?? -Infinity),
	// 			),
	// 		)
	// 	}

	// 	// Compute the boxes.
	// 	let {
	// 		chart_box,
	// 		xAxisLabelsBox,
	// 		xAxisTitleBox,
	// 		yAxisGridLineInfo,
	// 		yAxisLabelsBox,
	// 		yAxisTitleBox,
	// 	} = computeBoxes({
	// 		ctx,
	// 		height,
	// 		includeXAxisLabels: options.shouldDrawXAxisLabels ?? true,
	// 		includeXAxisTitle: xAxisTitle !== undefined,
	// 		includeYAxisLabels: options.shouldDrawYAxisLabels ?? true,
	// 		includeYAxisTitle: yAxisTitle !== undefined,
	// 		width,
	// 		xAxisGridLineInterval,
	// 		yAxisGridLineInterval,
	// 		yMax,
	// 		yMin,
	// 	})

	// 	// Compute the grid line info.
	// 	let xAxisGridLineInfo = computeXAxisGridLineInfo({
	// 		chartWidth: chart_box.w,
	// 		ctx,
	// 		xAxisGridLineInterval,
	// 		xMax,
	// 		xMin,
	// 	})

	// 	drawXAxisGridLines({
	// 		box: chart_box,
	// 		ctx,
	// 		xAxisGridLineInfo,
	// 	})

	// 	drawYAxisGridLines({
	// 		box: chart_box,
	// 		ctx,
	// 		yAxisGridLineInfo,
	// 	})

	// 	drawXAxis({
	// 		box: chart_box,
	// 		ctx,
	// 		yAxisGridLineInfo,
	// 	})

	// 	drawYAxis({
	// 		box: chart_box,
	// 		ctx,
	// 		xAxisGridLineInfo,
	// 	})

	// 	// Draw the X axis labels.
	// 	if (options.shouldDrawXAxisLabels ?? true) {
	// 		drawXAxisLabels({
	// 			box: xAxisLabelsBox,
	// 			ctx,
	// 			gridLineInfo: xAxisGridLineInfo,
	// 			labels,
	// 			width,
	// 		})
	// 	}

	// 	// Draw the Y axis labels.
	// 	if (options.shouldDrawYAxisLabels ?? true) {
	// 		drawYAxisLabels({
	// 			box: yAxisLabelsBox,
	// 			ctx,
	// 			fontSize: chartConfig.fontSize,
	// 			gridLineInfo: yAxisGridLineInfo,
	// 			height,
	// 		})
	// 	}

	// 	drawXAxisTitle({
	// 		box: xAxisTitleBox,
	// 		ctx,
	// 		title: xAxisTitle,
	// 	})

	// 	drawYAxisTitle({
	// 		box: yAxisTitleBox,
	// 		ctx,
	// 		title: yAxisTitle,
	// 	})

	// 	// Draw the lines.
	// 	data.forEach(series => {
	// 		drawLine({
	// 			chart_box,
	// 			ctx,
	// 			series,
	// 			xMax,
	// 			xMin,
	// 			yMax,
	// 			yMin,
	// 		})
	// 	})

	// 	let maxPointCount = Math.max(...data.map(series => series.data.length))
	// 	let shouldDrawPoints =
	// 		chart_box.w / maxPointCount > 2 * chartConfig.pointRadius

	// 	// Draw the points.
	// 	if (shouldDrawPoints) {
	// 		data.forEach(series => {
	// 			series.data.forEach(point => {
	// 				if (point.y === null) {
	// 					return
	// 				}
	// 				drawPoint({
	// 					chart_box,
	// 					color: series.color,
	// 					ctx,
	// 					point: { x: point.x, y: point.y },
	// 					pointStyle: series.pointStyle ?? PointStyle.Circle,
	// 					radius: chartConfig.pointRadius,
	// 					xMax,
	// 					xMin,
	// 					yMax,
	// 					yMin,
	// 				})
	// 			})
	// 		})
	// 	}

	// 	// Compute the hover regions.
	// 	let hasMultipleSeries = data.length > 1
	// 	data.forEach((series, seriesIndex) => {
	// 		series.data.forEach((point, pointIndex) => {
	// 			if (point.y === null) {
	// 				return
	// 			}
	// 			let pointPixels = pointToPixels({
	// 				chart_box,
	// 				point: { x: point.x, y: point.y },
	// 				xMax,
	// 				xMin,
	// 				yMax,
	// 				yMin,
	// 			})
	// 			let hoverRegion: HoverRegion<LineChartHoverRegionInfo> = {
	// 				distance: (mouseX: number, mouseY: number) => {
	// 					return (pointPixels.x - mouseX) ** 2 + (pointPixels.y - mouseY) ** 2
	// 				},
	// 				hitTest: (mouseX: number, mouseY: number) => {
	// 					return (
	// 						mouseX > pointPixels.x - chartConfig.tooltipTargetRadius &&
	// 						mouseX < pointPixels.x + chartConfig.tooltipTargetRadius &&
	// 						mouseY > pointPixels.y - chartConfig.tooltipTargetRadius &&
	// 						mouseY < pointPixels.y + chartConfig.tooltipTargetRadius
	// 					)
	// 				},
	// 				info: {
	// 					chart_box,
	// 					color: series.color,
	// 					point: { x: point.x, y: point.y },
	// 					pointLabel: labels?.[pointIndex],
	// 					pointValue: point.y,
	// 					seriesIndex,
	// 					seriesTitle: hasMultipleSeries ? series.title : undefined,
	// 					tooltipOriginPixels: { x: pointPixels.x, y: pointPixels.y },
	// 					xMax,
	// 					xMin,
	// 					yMax,
	// 					yMin,
	// 				},
	// 			}
	// 			hoverRegions.push(hoverRegion)
	// 		})
	// 	})

	// 	let overlayInfo: LineChartOverlayInfo = {
	// 		chart_box,
	// 		xMax,
	// 		xMin,
	// 		yMax,
	// 		yMin,
	// 	}

	// 	return { hoverRegions, overlayInfo }
}

struct DrawPointOptions {
	chart_box: Rect,
	color: String,
	ctx: CanvasRenderingContext2d,
	point: Point,
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
		point,
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

struct DrawLineOptions {
	chart_box: Rect,
	ctx: CanvasRenderingContext2d,
	series: LineChartSeries,
	x_max: f64,
	x_min: f64,
	y_max: f64,
	y_min: f64,
}

fn draw_line(options: DrawLineOptions) {
	// 	let {
	// 		chart_box,
	// 		ctx,
	// 		series,
	// 		series: { color },
	// 		xMax,
	// 		xMin,
	// 		yMax,
	// 		yMin,
	// 	} = options
	// 	if (series.lineStyle === LineStyle.Hidden) {
	// 		return
	// 	}
	// 	ctx.beginPath()
	// 	ctx.strokeStyle = color
	// 	if (options.series.lineStyle === LineStyle.Dashed) {
	// 		ctx.setLineDash([4, 4])
	// 	} else {
	// 		ctx.setLineDash([])
	// 	}
	// 	let data = series.data.filter((p: LineChartPoint): p is Point => p.y !== null)
	// 	if (data.length < 2) {
	// 		return
	// 	}
	// 	let firstPoint = data[0]
	// 	if (firstPoint === undefined) throw Error()
	// 	let firstPointPixels = pointToPixels({
	// 		chart_box,
	// 		point: firstPoint,
	// 		xMax,
	// 		xMin,
	// 		yMax,
	// 		yMin,
	// 	})
	// 	ctx.moveTo(firstPointPixels.x, firstPointPixels.y)
	// 	let cp1 = firstPoint
	// 	for (let i = 1; i < data.length - 1; i++) {
	// 		let previousPoint = data[i - 1]
	// 		if (previousPoint === undefined) throw Error()
	// 		let point = data[i]
	// 		if (point === undefined) throw Error()
	// 		let nextPoint = data[i + 1]
	// 		if (nextPoint === undefined) throw Error()
	// 		let [cp2, nextCp1] = interpolateSpline({
	// 			nextPoint,
	// 			point,
	// 			previousPoint,
	// 			tension: chartConfig.splineTension,
	// 		})
	// 		let cp1Pixels = pointToPixels({
	// 			chart_box,
	// 			point: cp1,
	// 			xMax,
	// 			xMin,
	// 			yMax,
	// 			yMin,
	// 		})
	// 		let cp2Pixels = pointToPixels({
	// 			chart_box,
	// 			point: cp2,
	// 			xMax,
	// 			xMin,
	// 			yMax,
	// 			yMin,
	// 		})
	// 		let pointPixels = pointToPixels({ chart_box, point, xMax, xMin, yMax, yMin })
	// 		ctx.bezierCurveTo(
	// 			cp1Pixels.x,
	// 			cp1Pixels.y,
	// 			cp2Pixels.x,
	// 			cp2Pixels.y,
	// 			pointPixels.x,
	// 			pointPixels.y,
	// 		)
	// 		cp1 = nextCp1
	// 	}
	// 	let lastPoint = data[data.length - 1]
	// 	if (lastPoint === undefined) throw Error()
	// 	let lastPointPixels = pointToPixels({
	// 		chart_box,
	// 		point: lastPoint,
	// 		xMax,
	// 		xMin,
	// 		yMax,
	// 		yMin,
	// 	})
	// 	let cp1Pixels = pointToPixels({
	// 		chart_box,
	// 		point: cp1,
	// 		xMax,
	// 		xMin,
	// 		yMax,
	// 		yMin,
	// 	})
	// 	ctx.bezierCurveTo(
	// 		cp1Pixels.x,
	// 		cp1Pixels.y,
	// 		lastPointPixels.x,
	// 		lastPointPixels.y,
	// 		lastPointPixels.x,
	// 		lastPointPixels.y,
	// 	)
	// 	ctx.stroke()
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

struct DrawLineChartOverlayOptions {
	active_hover_regions: Vec<ActiveHoverRegion<LineChartHoverRegionInfo>>,
	ctx: CanvasRenderingContext2d,
	info: LineChartOverlayInfo,
	overlay_div: HtmlElement,
}

fn draw_line_chart_overlay(options: DrawLineChartOverlayOptions) {
	// 	let {
	// 		activeHoverRegions,
	// 		ctx,
	// 		info: { chart_box, xMax, xMin, yMax, yMin },
	// 		overlayDiv,
	// 	} = options
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
	todo!()
	// 	let { chart_box, point, xMax, xMin, yMax, yMin } = options
	// 	return {
	// 		x:
	// 			chart_box.x +
	// 			(-xMin / (xMax - xMin)) * chart_box.w +
	// 			(point.x / (xMax - xMin)) * chart_box.w,
	// 		y:
	// 			chart_box.y +
	// 			chart_box.h -
	// 			(-yMin / (yMax - yMin)) * chart_box.h -
	// 			(point.y / (yMax - yMin)) * chart_box.h,
	// 	}
}
