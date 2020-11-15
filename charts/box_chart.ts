import { drawBarChartXAxisLabels } from "./bar_chart"
import { ActiveHoverRegion, HoverRegion, createChart } from "./chart"
import {
	Box,
	Point,
	computeBoxes,
	drawRoundedRect,
	drawXAxis,
	drawXAxisTitle,
	drawYAxisGridLines,
	drawYAxisLabels,
	drawYAxisTitle,
	formatNumber,
} from "./common"
import { chartColors, chartConfig } from "./config"
import { TooltipData, drawTooltip } from "./tooltip"

export type BoxChartOptions = {
	data: BoxChartData
	hideLegend?: boolean
	shouldDrawXAxisLabels?: boolean
	shouldDrawYAxisLabels?: boolean
	title?: string
	xAxisTitle?: string
	yAxisTitle?: string
	yMax?: number
	yMin?: number
}

export type BoxChartData = BoxChartSeries[]

export type BoxChartSeries = {
	color: string
	data: BoxChartPoint[]
	title: string
}

export type BoxChartPoint = {
	label: string
	x: number
	y: {
		max: number
		min: number
		p25: number
		p50: number
		p75: number
	} | null
}

export type BoxChartOverlayInfo = {
	chartBox: Box
}

export type BoxChartHoverRegionInfo = {
	color: string
	label: string
	name: string
	tooltipOriginPixels: Point
	value: number
}

export type DrawBoxChartOutput = {
	hoverRegions: Array<HoverRegion<BoxChartHoverRegionInfo>>
	overlayInfo: BoxChartOverlayInfo
}

export function createBoxChart(container: HTMLElement) {
	return createChart(container, drawBoxChart, drawBoxChartOverlay)
}

export function drawBoxChart(
	ctx: CanvasRenderingContext2D,
	options: BoxChartOptions,
): DrawBoxChartOutput {
	let { data, xAxisTitle, yAxisTitle } = options
	let width = ctx.canvas.clientWidth
	let height = ctx.canvas.clientHeight
	let hoverRegions: Array<HoverRegion<BoxChartHoverRegionInfo>> = []

	// Compute the bounds.
	let yMin: number
	if (options.yMin !== undefined) {
		yMin = options.yMin
	} else {
		yMin = Math.min(
			...data.flatMap(series => series.data.map(({ y }) => y?.min ?? Infinity)),
		)
		if (!isFinite(yMin)) {
			yMin = 0
		}
	}
	let yMax: number
	if (options.yMax !== undefined) {
		yMax = options.yMax
	} else {
		yMax = Math.max(
			...options.data.flatMap(series =>
				series.data.map(({ y }) => y?.max ?? -Infinity),
			),
		)
		if (!isFinite(yMax)) {
			yMax = yMin + 1
		}
	}

	let {
		chartBox,
		xAxisLabelsBox,
		xAxisTitleBox,
		yAxisGridLineInfo,
		yAxisLabelsBox,
		yAxisTitleBox,
	} = computeBoxes({
		ctx,
		height,
		includeXAxisLabels: options.shouldDrawXAxisLabels ?? true,
		includeXAxisTitle: xAxisTitle !== undefined,
		includeYAxisLabels: options.shouldDrawYAxisLabels ?? true,
		includeYAxisTitle: yAxisTitle !== undefined,
		width,
		yMax,
		yMin,
	})

	let categories = data[0].data.map(({ label }) => label)
	let boxGroupWidth =
		(chartBox.w - chartConfig.barGroupGap * (categories.length + 1)) /
		categories.length

	// Draw the X axis labels.
	if (options.shouldDrawXAxisLabels ?? true) {
		drawBarChartXAxisLabels({
			barGroupGap: chartConfig.barGroupGap,
			box: xAxisLabelsBox,
			categories,
			ctx,
			groupWidth: boxGroupWidth,
			width,
		})
	}

	drawYAxisGridLines({
		box: chartBox,
		ctx,
		yAxisGridLineInfo,
	})

	drawXAxis({
		box: chartBox,
		ctx,
		yAxisGridLineInfo,
	})

	// Draw the Y axis labels.
	if (options.shouldDrawYAxisLabels ?? true) {
		drawYAxisLabels({
			box: yAxisLabelsBox,
			ctx,
			fontSize: chartConfig.fontSize,
			gridLineInfo: yAxisGridLineInfo,
			height,
		})
	}

	drawXAxisTitle({
		box: xAxisTitleBox,
		ctx,
		title: xAxisTitle,
	})

	drawYAxisTitle({
		box: yAxisTitleBox,
		ctx,
		title: yAxisTitle,
	})

	// Draw the boxes.
	data.forEach((series, seriesIndex) => {
		series.data.forEach((point, pointIndex) => {
			let output = drawBox({
				boxGap: chartConfig.barGap,
				boxGroupGap: chartConfig.barGroupGap,
				boxGroupWidth,
				chartBox,
				ctx,
				data,
				point,
				pointIndex,
				series,
				seriesIndex,
				yMax,
				yMin,
			})
			hoverRegions.push(...output.hoverRegions)
		})
	})

	let overlayInfo: BoxChartOverlayInfo = {
		chartBox,
	}

	return { hoverRegions, overlayInfo }
}

type DrawBoxChartOverlayOptions = {
	activeHoverRegions: Array<ActiveHoverRegion<BoxChartHoverRegionInfo>>
	ctx: CanvasRenderingContext2D
	info: BoxChartOverlayInfo
	overlayDiv: HTMLElement
}

export function drawBoxChartOverlay(options: DrawBoxChartOverlayOptions) {
	let {
		activeHoverRegions,
		ctx,
		info: { chartBox },
		overlayDiv,
	} = options
	let tooltips: TooltipData[] = []
	let boxPointIndexForName: { [key: string]: number } = {
		max: 4,
		median: 2,
		min: 0,
		p25: 1,
		p75: 3,
	}
	activeHoverRegions.sort((activeHoverRegionA, activeHoverRegionB) => {
		let boxPointIndexA = boxPointIndexForName[activeHoverRegionA.info.name]
		let boxPointIndexB = boxPointIndexForName[activeHoverRegionB.info.name]
		return boxPointIndexA > boxPointIndexB ? -1 : 1
	})
	for (let i = 0; i < activeHoverRegions.length; i++) {
		let activeHoverRegion = activeHoverRegions[i]
		let color = activeHoverRegion.info.color
		let x = activeHoverRegion.info.label
		let name = activeHoverRegion.info.name
		let value = formatNumber(activeHoverRegion.info.value)
		let y = `${name} = ${value}`
		let text = `(${x}, ${y})`
		tooltips.push({
			color,
			text,
		})
	}
	for (let activeHoverRegion of activeHoverRegions) {
		drawLine({
			color: chartColors.current.crosshairsColor,
			ctx,
			dashed: true,
			end: {
				x: chartBox.x + chartBox.w,
				y: activeHoverRegion.info.tooltipOriginPixels.y,
			},
			start: { x: chartBox.x, y: activeHoverRegion.info.tooltipOriginPixels.y },
		})
	}
	if (tooltips.length > 0) {
		let origin =
			activeHoverRegions[activeHoverRegions.length - 1].info.tooltipOriginPixels
		drawTooltip({
			centerHorizontal: true,
			container: overlayDiv,
			origin,
			values: tooltips,
		})
	}
}

type DrawBoxOptions = {
	boxGap: number
	boxGroupGap: number
	boxGroupWidth: number
	chartBox: Box
	ctx: CanvasRenderingContext2D
	data: BoxChartData
	point: BoxChartPoint
	pointIndex: number
	series: BoxChartSeries
	seriesIndex: number
	yMax: number
	yMin: number
}

type DrawBoxOutput = {
	hoverRegions: Array<HoverRegion<BoxChartHoverRegionInfo>>
}

function drawBox(options: DrawBoxOptions): DrawBoxOutput {
	let {
		boxGap,
		boxGroupGap,
		boxGroupWidth,
		chartBox,
		ctx,
		data,
		point,
		pointIndex,
		series,
		seriesIndex,
		yMax,
		yMin,
	} = options
	let hoverRegions: Array<HoverRegion<BoxChartHoverRegionInfo>> = []

	// Ignore boxes with null values.
	if (!point.y) {
		return { hoverRegions }
	}

	let boxWidth =
		boxGroupWidth / data.length - chartConfig.barGap * (data.length - 1)
	let x =
		chartBox.x +
		(boxGroupGap + (boxGroupGap + boxGroupWidth) * pointIndex) +
		(boxGap + boxWidth) * seriesIndex

	let whiskerTipWidth = boxWidth / 10
	let lineWidth = 2
	let valueToPixels = (value: number) =>
		chartBox.y +
		chartBox.h -
		(-yMin / (yMax - yMin)) * chartBox.h -
		(value / (yMax - yMin)) * chartBox.h

	// Draw the box.
	let box = {
		h: (Math.abs(point.y.p75 - point.y.p25) / (yMax - yMin)) * chartBox.h,
		w: boxWidth,
		x,
		y: valueToPixels(Math.max(point.y.p25, point.y.p75)),
	}
	drawRoundedRect({
		box,
		ctx,
		fillColor: series.color + "af",
		radius: Math.min(
			Math.abs(box.h / 2),
			Math.abs(box.w / 6),
			chartConfig.maxCornerRadius,
		),
		strokeColor: series.color,
		strokeWidth: chartConfig.barStrokeWidth,
	})

	// Create a clip path so the median line will not overflow the box.
	ctx.save()
	drawRoundedRect({
		box,
		ctx,
		radius: Math.min(
			Math.abs(box.h / 2),
			Math.abs(box.w / 6),
			chartConfig.maxCornerRadius,
		),
	})
	ctx.clip()
	// Draw the median line.
	let medianBox = {
		h: lineWidth,
		w: boxWidth,
		x,
		y: valueToPixels(point.y.p50),
	}
	drawLine({
		color: series.color,
		ctx,
		end: { x: medianBox.x + medianBox.w, y: medianBox.y },
		lineWidth,
		start: { x: medianBox.x, y: medianBox.y },
	})
	hoverRegions.push(
		boxChartHoverRegion({
			box: medianBox,
			color: series.color,
			label: point.label,
			name: "median",
			tooltipOriginPixels: { ...medianBox, x: x + boxWidth / 2 },
			value: point.y.p50,
		}),
	)
	ctx.restore()

	// Draw the min line.
	drawLine({
		color: series.color,
		ctx,
		end: {
			x: x + boxWidth / 2,
			y: valueToPixels(point.y.min),
		},
		lineWidth,
		start: {
			x: x + boxWidth / 2,
			y: valueToPixels(point.y.p25),
		},
	})
	let minWhiskerTipBox = {
		h: lineWidth,
		w: whiskerTipWidth,
		x: x + boxWidth / 2 - whiskerTipWidth / 2,
		y: valueToPixels(point.y.min),
	}
	drawLine({
		color: series.color,
		ctx,
		end: {
			x: minWhiskerTipBox.x + minWhiskerTipBox.w,
			y: minWhiskerTipBox.y,
		},
		lineCap: "round",
		lineWidth,
		start: { x: minWhiskerTipBox.x, y: minWhiskerTipBox.y },
	})
	hoverRegions.push(
		boxChartHoverRegion({
			box: minWhiskerTipBox,
			color: series.color,
			label: point.label,
			name: "min",
			tooltipOriginPixels: { ...minWhiskerTipBox, x: x + boxWidth / 2 },
			value: point.y.min,
		}),
	)

	// Draw the max line.
	drawLine({
		color: series.color,
		ctx,
		end: {
			x: x + boxWidth / 2,
			y: valueToPixels(point.y.max),
		},
		lineWidth,
		start: {
			x: x + boxWidth / 2,
			y: valueToPixels(point.y.p75),
		},
	})
	let maxWhiskerTipBox = {
		h: lineWidth,
		w: whiskerTipWidth,
		x: x + boxWidth / 2 - whiskerTipWidth / 2,
		y: valueToPixels(point.y.max),
	}
	drawLine({
		color: series.color,
		ctx,
		end: {
			x: maxWhiskerTipBox.x + maxWhiskerTipBox.w,
			y: maxWhiskerTipBox.y,
		},
		lineCap: "round",
		lineWidth,
		start: { x: maxWhiskerTipBox.x, y: maxWhiskerTipBox.y },
	})
	hoverRegions.push(
		boxChartHoverRegion({
			box: maxWhiskerTipBox,
			color: series.color,
			label: point.label,
			name: "max",
			tooltipOriginPixels: {
				...maxWhiskerTipBox,
				x: x + boxWidth / 2,
			},
			value: point.y.max,
		}),
	)

	// Register the p25 hit region.
	let p25Box = {
		h: lineWidth,
		w: boxWidth,
		x,
		y: valueToPixels(point.y.p25),
	}
	hoverRegions.push(
		boxChartHoverRegion({
			box: p25Box,
			color: series.color,
			label: point.label,
			name: "p25",
			tooltipOriginPixels: {
				...p25Box,
				x: x + boxWidth / 2,
			},
			value: point.y.p25,
		}),
	)

	// Register the p75 hit region.
	let p75Box = {
		h: lineWidth,
		w: boxWidth,
		x,
		y: valueToPixels(point.y.p75),
	}
	hoverRegions.push(
		boxChartHoverRegion({
			box: p75Box,
			color: series.color,
			label: point.label,
			name: "p75",
			tooltipOriginPixels: {
				...p75Box,
				x: x + boxWidth / 2,
			},
			value: point.y.p75,
		}),
	)

	return { hoverRegions }
}

type RegisterBoxChartHoverRegionOptions = {
	box: Box
	color: string
	label: string
	name: string
	tooltipOriginPixels: Box
	value: number
}

function boxChartHoverRegion(
	options: RegisterBoxChartHoverRegionOptions,
): HoverRegion<BoxChartHoverRegionInfo> {
	let { box, color, label, name, tooltipOriginPixels, value } = options
	return {
		distance: (mouseX: number, mouseY: number) => {
			return (box.x - mouseX) ** 2 + (box.y - mouseY) ** 2
		},
		hitTest: (mouseX: number, mouseY: number) => {
			return (
				mouseY < box.y + box.h + chartConfig.tooltipTargetRadius &&
				mouseY > box.y - box.h - chartConfig.tooltipTargetRadius &&
				mouseX > box.x - chartConfig.tooltipTargetRadius &&
				mouseX < box.x + box.w + chartConfig.tooltipTargetRadius
			)
		},
		info: {
			color,
			label,
			name,
			tooltipOriginPixels,
			value,
		},
	}
}

type DrawLineOptions = {
	color?: string
	ctx: CanvasRenderingContext2D
	dashed?: boolean
	end: Point
	lineCap?: CanvasLineCap
	lineWidth?: number
	start: Point
}

export function drawLine(options: DrawLineOptions) {
	let { color, ctx, dashed, end, lineCap, lineWidth, start } = options
	lineWidth = lineWidth ?? 1
	dashed = dashed ?? false
	lineCap = lineCap ?? "butt"
	ctx.save()
	if (dashed) {
		ctx.setLineDash([4, 4])
	}
	if (color) {
		ctx.strokeStyle = color
	}
	ctx.lineWidth = lineWidth
	ctx.lineCap = lineCap
	ctx.beginPath()
	ctx.moveTo(start.x, start.y)
	ctx.lineTo(end.x, end.y)
	ctx.stroke()
	ctx.restore()
}
