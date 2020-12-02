use crate::{
	chart::{ActiveHoverRegion, DrawChartOptions, HoverRegion},
	common::{Point, Rect},
};
use wasm_bindgen::JsValue;
use web_sys::*;

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

pub struct DrawBoxChartOutput {
	hover_regions: Vec<HoverRegion<BoxChartHoverRegionInfo>>,
	overlay_info: BoxChartOverlayInfo,
}

fn draw_box_chart(options: DrawChartOptions<BoxChartOptions>) -> DrawBoxChartOutput {
	todo!()
	// 	let { series: data, xAxisTitle, yAxisTitle } = options
	// 	let width = ctx.canvas.clientWidth
	// 	let height = ctx.canvas.clientHeight
	// 	let hoverRegions: Array<HoverRegion<BoxChartHoverRegionInfo>> = []

	// 	// Compute the bounds.
	// 	let yMin: number
	// 	if (options.yMin !== undefined) {
	// 		yMin = options.yMin
	// 	} else {
	// 		yMin = Math.min(
	// 			...data.flatMap(series => series.data.map(({ y }) => y?.min ?? Infinity)),
	// 		)
	// 		if (!isFinite(yMin)) {
	// 			yMin = 0
	// 		}
	// 	}
	// 	let yMax: number
	// 	if (options.yMax !== undefined) {
	// 		yMax = options.yMax
	// 	} else {
	// 		yMax = Math.max(
	// 			...options.series.flatMap(series =>
	// 				series.data.map(({ y }) => y?.max ?? -Infinity),
	// 			),
	// 		)
	// 		if (!isFinite(yMax)) {
	// 			yMax = yMin + 1
	// 		}
	// 	}

	// 	let {
	// 		chartBox,
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
	// 		yMax,
	// 		yMin,
	// 	})

	// 	if (data[0] === undefined) throw Error()
	// 	let categories = data[0].data.map(({ label }) => label)
	// 	let boxGroupWidth =
	// 		(chartBox.w - chartConfig.barGroupGap * (categories.length + 1)) /
	// 		categories.length

	// 	// Draw the X axis labels.
	// 	if (options.shouldDrawXAxisLabels ?? true) {
	// 		drawBarChartXAxisLabels({
	// 			barGroupGap: chartConfig.barGroupGap,
	// 			box: xAxisLabelsBox,
	// 			categories,
	// 			ctx,
	// 			groupWidth: boxGroupWidth,
	// 			width,
	// 		})
	// 	}

	// 	drawYAxisGridLines({
	// 		box: chartBox,
	// 		ctx,
	// 		yAxisGridLineInfo,
	// 	})

	// 	drawXAxis({
	// 		box: chartBox,
	// 		ctx,
	// 		yAxisGridLineInfo,
	// 	})

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

	// 	// Draw the boxes.
	// 	data.forEach((series, seriesIndex) => {
	// 		series.data.forEach((point, pointIndex) => {
	// 			let output = drawBox({
	// 				boxGap: chartConfig.barGap,
	// 				boxGroupGap: chartConfig.barGroupGap,
	// 				boxGroupWidth,
	// 				chartBox,
	// 				ctx,
	// 				data,
	// 				point,
	// 				pointIndex,
	// 				series,
	// 				seriesIndex,
	// 				yMax,
	// 				yMin,
	// 			})
	// 			hoverRegions.push(...output.hoverRegions)
	// 		})
	// 	})

	// 	let overlayInfo: BoxChartOverlayInfo = {
	// 		chartBox,
	// 	}

	// 	return { hoverRegions, overlayInfo }
}

struct DrawBoxChartOverlayOptions {
	active_hover_regions: Vec<ActiveHoverRegion<BoxChartHoverRegionInfo>>,
	ctx: CanvasRenderingContext2d,
	info: BoxChartOverlayInfo,
	overlay_div: HtmlElement,
}

fn draw_box_chart_overlay(options: DrawBoxChartOverlayOptions) {
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

struct DrawBoxOptions {
	box_gap: f64,
	box_group_gap: f64,
	box_group_width: f64,
	chart_box: Rect,
	ctx: CanvasRenderingContext2d,
	data: Vec<BoxChartSeries>,
	point: BoxChartPoint,
	point_index: f64,
	series: BoxChartSeries,
	series_index: f64,
	y_max: f64,
	y_min: f64,
}

struct DrawBoxOutput {
	hover_regions: Vec<HoverRegion<BoxChartHoverRegionInfo>>,
}

fn draw_box(options: DrawBoxOptions) -> DrawBoxOutput {
	todo!()
	// 	let {
	// 		boxGap,
	// 		boxGroupGap,
	// 		boxGroupWidth,
	// 		chartBox,
	// 		ctx,
	// 		data,
	// 		point,
	// 		pointIndex,
	// 		series,
	// 		seriesIndex,
	// 		yMax,
	// 		yMin,
	// 	} = options
	// 	let hoverRegions: Array<HoverRegion<BoxChartHoverRegionInfo>> = []

	// 	// Ignore boxes with null values.
	// 	if (!point.y) {
	// 		return { hoverRegions }
	// 	}

	// 	let boxWidth =
	// 		boxGroupWidth / data.length - chartConfig.barGap * (data.length - 1)
	// 	let x =
	// 		chartBox.x +
	// 		(boxGroupGap + (boxGroupGap + boxGroupWidth) * pointIndex) +
	// 		(boxGap + boxWidth) * seriesIndex

	// 	let whiskerTipWidth = boxWidth / 10
	// 	let lineWidth = 2
	// 	let valueToPixels = (value: number) =>
	// 		chartBox.y +
	// 		chartBox.h -
	// 		(-yMin / (yMax - yMin)) * chartBox.h -
	// 		(value / (yMax - yMin)) * chartBox.h

	// 	// Draw the box.
	// 	let box = {
	// 		h: (Math.abs(point.y.p75 - point.y.p25) / (yMax - yMin)) * chartBox.h,
	// 		w: boxWidth,
	// 		x,
	// 		y: valueToPixels(Math.max(point.y.p25, point.y.p75)),
	// 	}
	// 	drawRoundedRect({
	// 		box,
	// 		ctx,
	// 		fillColor: series.color + "af",
	// 		radius: Math.min(
	// 			Math.abs(box.h / 2),
	// 			Math.abs(box.w / 6),
	// 			chartConfig.maxCornerRadius,
	// 		),
	// 		strokeColor: series.color,
	// 		strokeWidth: chartConfig.barStrokeWidth,
	// 	})

	// 	// Create a clip path so the median line will not overflow the box.
	// 	ctx.save()
	// 	drawRoundedRect({
	// 		box,
	// 		ctx,
	// 		radius: Math.min(
	// 			Math.abs(box.h / 2),
	// 			Math.abs(box.w / 6),
	// 			chartConfig.maxCornerRadius,
	// 		),
	// 	})
	// 	ctx.clip()
	// 	// Draw the median line.
	// 	let medianBox = {
	// 		h: lineWidth,
	// 		w: boxWidth,
	// 		x,
	// 		y: valueToPixels(point.y.p50),
	// 	}
	// 	drawLine({
	// 		color: series.color,
	// 		ctx,
	// 		end: { x: medianBox.x + medianBox.w, y: medianBox.y },
	// 		lineWidth,
	// 		start: { x: medianBox.x, y: medianBox.y },
	// 	})
	// 	hoverRegions.push(
	// 		boxChartHoverRegion({
	// 			box: medianBox,
	// 			color: series.color,
	// 			label: point.label,
	// 			name: "median",
	// 			tooltipOriginPixels: { ...medianBox, x: x + boxWidth / 2 },
	// 			value: point.y.p50,
	// 		}),
	// 	)
	// 	ctx.restore()

	// 	// Draw the min line.
	// 	drawLine({
	// 		color: series.color,
	// 		ctx,
	// 		end: {
	// 			x: x + boxWidth / 2,
	// 			y: valueToPixels(point.y.min),
	// 		},
	// 		lineWidth,
	// 		start: {
	// 			x: x + boxWidth / 2,
	// 			y: valueToPixels(point.y.p25),
	// 		},
	// 	})
	// 	let minWhiskerTipBox = {
	// 		h: lineWidth,
	// 		w: whiskerTipWidth,
	// 		x: x + boxWidth / 2 - whiskerTipWidth / 2,
	// 		y: valueToPixels(point.y.min),
	// 	}
	// 	drawLine({
	// 		color: series.color,
	// 		ctx,
	// 		end: {
	// 			x: minWhiskerTipBox.x + minWhiskerTipBox.w,
	// 			y: minWhiskerTipBox.y,
	// 		},
	// 		lineCap: "round",
	// 		lineWidth,
	// 		start: { x: minWhiskerTipBox.x, y: minWhiskerTipBox.y },
	// 	})
	// 	hoverRegions.push(
	// 		boxChartHoverRegion({
	// 			box: minWhiskerTipBox,
	// 			color: series.color,
	// 			label: point.label,
	// 			name: "min",
	// 			tooltipOriginPixels: { ...minWhiskerTipBox, x: x + boxWidth / 2 },
	// 			value: point.y.min,
	// 		}),
	// 	)

	// 	// Draw the max line.
	// 	drawLine({
	// 		color: series.color,
	// 		ctx,
	// 		end: {
	// 			x: x + boxWidth / 2,
	// 			y: valueToPixels(point.y.max),
	// 		},
	// 		lineWidth,
	// 		start: {
	// 			x: x + boxWidth / 2,
	// 			y: valueToPixels(point.y.p75),
	// 		},
	// 	})
	// 	let maxWhiskerTipBox = {
	// 		h: lineWidth,
	// 		w: whiskerTipWidth,
	// 		x: x + boxWidth / 2 - whiskerTipWidth / 2,
	// 		y: valueToPixels(point.y.max),
	// 	}
	// 	drawLine({
	// 		color: series.color,
	// 		ctx,
	// 		end: {
	// 			x: maxWhiskerTipBox.x + maxWhiskerTipBox.w,
	// 			y: maxWhiskerTipBox.y,
	// 		},
	// 		lineCap: "round",
	// 		lineWidth,
	// 		start: { x: maxWhiskerTipBox.x, y: maxWhiskerTipBox.y },
	// 	})
	// 	hoverRegions.push(
	// 		boxChartHoverRegion({
	// 			box: maxWhiskerTipBox,
	// 			color: series.color,
	// 			label: point.label,
	// 			name: "max",
	// 			tooltipOriginPixels: {
	// 				...maxWhiskerTipBox,
	// 				x: x + boxWidth / 2,
	// 			},
	// 			value: point.y.max,
	// 		}),
	// 	)

	// 	// Register the p25 hit region.
	// 	let p25Box = {
	// 		h: lineWidth,
	// 		w: boxWidth,
	// 		x,
	// 		y: valueToPixels(point.y.p25),
	// 	}
	// 	hoverRegions.push(
	// 		boxChartHoverRegion({
	// 			box: p25Box,
	// 			color: series.color,
	// 			label: point.label,
	// 			name: "p25",
	// 			tooltipOriginPixels: {
	// 				...p25Box,
	// 				x: x + boxWidth / 2,
	// 			},
	// 			value: point.y.p25,
	// 		}),
	// 	)

	// 	// Register the p75 hit region.
	// 	let p75Box = {
	// 		h: lineWidth,
	// 		w: boxWidth,
	// 		x,
	// 		y: valueToPixels(point.y.p75),
	// 	}
	// 	hoverRegions.push(
	// 		boxChartHoverRegion({
	// 			box: p75Box,
	// 			color: series.color,
	// 			label: point.label,
	// 			name: "p75",
	// 			tooltipOriginPixels: {
	// 				...p75Box,
	// 				x: x + boxWidth / 2,
	// 			},
	// 			value: point.y.p75,
	// 		}),
	// 	)

	// 	return { hoverRegions }
}

struct BoxChartHoverRegionOptions {
	rect: Rect,
	color: String,
	label: String,
	name: String,
	tooltip_origin_pixels: Rect,
	value: f64,
}

fn box_chart_hover_region(
	options: BoxChartHoverRegionOptions,
) -> HoverRegion<BoxChartHoverRegionInfo> {
	todo!()
	// let { box, color, label, name, tooltipOriginPixels, value } = options
	// return {
	// 	distance: (mouseX: number, mouseY: number) => {
	// 		return (box.x - mouseX) ** 2 + (box.y - mouseY) ** 2
	// 	},
	// 	hitTest: (mouseX: number, mouseY: number) => {
	// 		return (
	// 			mouseY < box.y + box.h + chartConfig.tooltipTargetRadius &&
	// 			mouseY > box.y - box.h - chartConfig.tooltipTargetRadius &&
	// 			mouseX > box.x - chartConfig.tooltipTargetRadius &&
	// 			mouseX < box.x + box.w + chartConfig.tooltipTargetRadius
	// 		)
	// 	},
	// 	info: {
	// 		color,
	// 		label,
	// 		name,
	// 		tooltipOriginPixels,
	// 		value,
	// 	},
	// }
}

struct DrawLineOptions {
	color: Option<String>,
	ctx: CanvasRenderingContext2d,
	dashed: Option<bool>,
	end: Point,
	line_cap: Option<String>,
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
		ctx.set_stroke_style(&color.into());
	}
	ctx.set_line_width(line_width);
	ctx.set_line_cap(line_cap);
	ctx.begin_path();
	ctx.move_to(start.x, start.y);
	ctx.line_to(end.x, end.y);
	ctx.stroke();
	ctx.restore();
}
