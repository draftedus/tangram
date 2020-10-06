import { ActiveHoverRegion, HoverRegion, createChart } from './chart'
import {
	Box,
	GridLineInterval,
	Point,
	computeBoxes,
	computeXAxisGridLineInfo,
	drawXAxis,
	drawXAxisGridLines,
	drawXAxisLabels,
	drawXAxisTitle,
	drawYAxis,
	drawYAxisGridLines,
	drawYAxisLabels,
	drawYAxisTitle,
	formatNumber,
} from './common'
import { chartColors, chartConfig } from './config'
import { TooltipData, drawTooltip } from './tooltip'

export type LineChartOptions = {
	data: LineChartData
	hideLegend?: boolean
	labels?: string[]
	shouldDrawXAxisLabels?: boolean
	shouldDrawYAxisLabels?: boolean
	title?: string
	xAxisGridLineInterval?: GridLineInterval
	xAxisTitle?: string
	xMax?: number
	xMin?: number
	yAxisGridLineInterval?: GridLineInterval
	yAxisTitle?: string
	yMax?: number
	yMin?: number
}

export type LineChartData = LineChartSeries[]

export type LineChartSeries = {
	color: string
	data: LineChartPoint[]
	lineStyle?: LineStyle
	pointStyle?: PointStyle
	title: string
}

export type LineChartPoint = {
	x: number
	y: number | null
}

export enum LineStyle {
	Hidden,
	Solid,
	Dashed,
}

export enum PointStyle {
	Hidden,
	Circle,
}

export type LineChartOverlayInfo = {
	chartBox: Box
	xMax: number
	xMin: number
	yMax: number
	yMin: number
}

export type LineChartHoverRegionInfo = {
	chartBox: Box
	color: string
	label?: string
	point: Point
	seriesIndex: number
	tooltipOriginPixels: Point
	xMax: number
	xMin: number
	yMax: number
	yMin: number
}

export type DrawLineChartOutput = {
	hoverRegions: Array<HoverRegion<LineChartHoverRegionInfo>>
	overlayInfo: LineChartOverlayInfo
}

export function createLineChart(container: HTMLElement) {
	return createChart(container, drawLineChart, drawLineChartOverlay)
}

export function drawLineChart(
	ctx: CanvasRenderingContext2D,
	options: LineChartOptions,
): DrawLineChartOutput {
	let {
		data,
		labels,
		xAxisGridLineInterval,
		xAxisTitle,
		yAxisGridLineInterval,
		yAxisTitle,
	} = options
	let width = ctx.canvas.clientWidth
	let height = ctx.canvas.clientHeight
	let hoverRegions: Array<HoverRegion<LineChartHoverRegionInfo>> = []

	// Compute the bounds.
	let xMin: number
	if (options.xMin !== undefined) {
		xMin = options.xMin
	} else {
		xMin = Math.min(
			...options.data.flatMap(series => series.data.map(({ x }) => x)),
		)
	}
	let xMax: number
	if (options.xMax !== undefined) {
		xMax = options.xMax
	} else {
		xMax = Math.max(
			...options.data.flatMap(series => series.data.map(({ x }) => x)),
		)
	}
	let yMin: number
	if (options.yMin !== undefined) {
		yMin = options.yMin
	} else {
		yMin = Math.min(
			...options.data.flatMap(series => series.data.map(p => p.y ?? Infinity)),
		)
	}
	let yMax: number
	if (options.yMax !== undefined) {
		yMax = options.yMax
	} else {
		yMax = Math.max(
			...options.data.flatMap(series => series.data.map(p => p.y ?? -Infinity)),
		)
	}

	// Compute the boxes.
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
		xAxisGridLineInterval,
		yAxisGridLineInterval,
		yMax,
		yMin,
	})

	// Compute the grid line info.
	let xAxisGridLineInfo = computeXAxisGridLineInfo({
		chartWidth: chartBox.w,
		ctx,
		xAxisGridLineInterval,
		xMax,
		xMin,
	})

	drawXAxisGridLines({
		box: chartBox,
		ctx,
		xAxisGridLineInfo,
	})

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

	drawYAxis({
		box: chartBox,
		ctx,
		xAxisGridLineInfo,
	})

	// Draw the X axis labels.
	if (options.shouldDrawXAxisLabels ?? true) {
		drawXAxisLabels({
			box: xAxisLabelsBox,
			ctx,
			gridLineInfo: xAxisGridLineInfo,
			labels,
			width,
		})
	}

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

	// Draw the lines.
	data.forEach(series => {
		drawLine({
			chartBox,
			ctx,
			series,
			xMax,
			xMin,
			yMax,
			yMin,
		})
	})

	let maxPointCount = Math.max(...data.map(series => series.data.length))
	let shouldDrawPoints =
		chartBox.w / maxPointCount > 2 * chartConfig.pointRadius

	// Draw the points.
	if (shouldDrawPoints) {
		data.forEach(series => {
			series.data.forEach(point => {
				if (point.y === null) {
					return
				}
				drawPoint({
					chartBox,
					color: series.color,
					ctx,
					point: { x: point.x, y: point.y },
					pointStyle: series.pointStyle ?? PointStyle.Circle,
					radius: chartConfig.pointRadius,
					xMax,
					xMin,
					yMax,
					yMin,
				})
			})
		})
	}

	// Compute the hover regions.
	data.forEach((series, seriesIndex) => {
		series.data.forEach((point, pointIndex) => {
			if (point.y === null) {
				return
			}
			let pointPixels = pointToPixels({
				chartBox,
				point: { x: point.x, y: point.y },
				xMax,
				xMin,
				yMax,
				yMin,
			})
			let hoverRegion: HoverRegion<LineChartHoverRegionInfo> = {
				distance: (mouseX: number, mouseY: number) => {
					return (pointPixels.x - mouseX) ** 2 + (pointPixels.y - mouseY) ** 2
				},
				hitTest: (mouseX: number, mouseY: number) => {
					return (
						mouseX > pointPixels.x - chartConfig.tooltipTargetRadius &&
						mouseX < pointPixels.x + chartConfig.tooltipTargetRadius &&
						mouseY > pointPixels.y - chartConfig.tooltipTargetRadius &&
						mouseY < pointPixels.y + chartConfig.tooltipTargetRadius
					)
				},
				info: {
					chartBox,
					color: series.color,
					label: labels?.[pointIndex],
					point: { x: point.x, y: point.y },
					seriesIndex,
					tooltipOriginPixels: { x: pointPixels.x, y: pointPixels.y },
					xMax,
					xMin,
					yMax,
					yMin,
				},
			}
			hoverRegions.push(hoverRegion)
		})
	})

	let overlayInfo: LineChartOverlayInfo = {
		chartBox,
		xMax,
		xMin,
		yMax,
		yMin,
	}

	return { hoverRegions, overlayInfo }
}

type DrawPointOptions = {
	chartBox: Box
	color: string
	ctx: CanvasRenderingContext2D
	point: Point
	pointStyle: PointStyle
	radius: number
	xMax: number
	xMin: number
	yMax: number
	yMin: number
}

function drawPoint(options: DrawPointOptions) {
	let {
		chartBox,
		color,
		ctx,
		point,
		pointStyle,
		radius,
		xMax,
		xMin,
		yMax,
		yMin,
	} = options
	if (pointStyle === PointStyle.Hidden) {
		return
	}
	let pointPixels = pointToPixels({ chartBox, point, xMax, xMin, yMax, yMin })
	ctx.beginPath()
	ctx.fillStyle = color
	ctx.arc(pointPixels.x, pointPixels.y, radius, 0, 2 * Math.PI)
	ctx.fill()
}

type DrawLineOptions = {
	chartBox: Box
	ctx: CanvasRenderingContext2D
	series: LineChartSeries
	xMax: number
	xMin: number
	yMax: number
	yMin: number
}

function drawLine(options: DrawLineOptions) {
	let {
		chartBox,
		ctx,
		series,
		series: { color },
		xMax,
		xMin,
		yMax,
		yMin,
	} = options
	if (series.lineStyle === LineStyle.Hidden) {
		return
	}
	ctx.beginPath()
	ctx.strokeStyle = color
	if (options.series.lineStyle === LineStyle.Dashed) {
		ctx.setLineDash([4, 4])
	} else {
		ctx.setLineDash([])
	}
	let data = series.data.filter((p: LineChartPoint): p is Point => p.y !== null)
	if (data.length < 2) {
		return
	}
	let firstPoint = data[0]
	let firstPointPixels = pointToPixels({
		chartBox,
		point: firstPoint,
		xMax,
		xMin,
		yMax,
		yMin,
	})
	ctx.moveTo(firstPointPixels.x, firstPointPixels.y)
	let cp1 = firstPoint
	for (let i = 1; i < data.length - 1; i++) {
		let previousPoint = data[i - 1]
		let point = data[i]
		let nextPoint = data[i + 1]
		let [cp2, nextCp1] = interpolateSpline({
			nextPoint,
			point,
			previousPoint,
			tension: chartConfig.splineTension,
		})
		let cp1Pixels = pointToPixels({
			chartBox,
			point: cp1,
			xMax,
			xMin,
			yMax,
			yMin,
		})
		let cp2Pixels = pointToPixels({
			chartBox,
			point: cp2,
			xMax,
			xMin,
			yMax,
			yMin,
		})
		let pointPixels = pointToPixels({ chartBox, point, xMax, xMin, yMax, yMin })
		ctx.bezierCurveTo(
			cp1Pixels.x,
			cp1Pixels.y,
			cp2Pixels.x,
			cp2Pixels.y,
			pointPixels.x,
			pointPixels.y,
		)
		cp1 = nextCp1
	}
	let lastPoint = data[data.length - 1]
	let lastPointPixels = pointToPixels({
		chartBox,
		point: lastPoint,
		xMax,
		xMin,
		yMax,
		yMin,
	})
	let cp1Pixels = pointToPixels({
		chartBox,
		point: cp1,
		xMax,
		xMin,
		yMax,
		yMin,
	})
	ctx.bezierCurveTo(
		cp1Pixels.x,
		cp1Pixels.y,
		lastPointPixels.x,
		lastPointPixels.y,
		lastPointPixels.x,
		lastPointPixels.y,
	)
	ctx.stroke()
}

type InterpolateSplineOptions = {
	nextPoint: Point
	point: Point
	previousPoint: Point
	tension: number
}

function interpolateSpline(options: InterpolateSplineOptions): [Point, Point] {
	let { nextPoint, point, previousPoint, tension } = options
	let d01 = Math.sqrt(
		(point.x - previousPoint.x) ** 2 + (point.y - previousPoint.y) ** 2,
	)
	let d12 = Math.sqrt(
		(point.x - nextPoint.x) ** 2 + (point.y - nextPoint.y) ** 2,
	)
	let m01 = (tension * d01) / (d01 + d12)
	let m12 = (tension * d12) / (d01 + d12)
	let cp1 = {
		x: point.x - m01 * (nextPoint.x - previousPoint.x),
		y: point.y - m01 * (nextPoint.y - previousPoint.y),
	}
	let cp2 = {
		x: point.x + m12 * (nextPoint.x - previousPoint.x),
		y: point.y + m12 * (nextPoint.y - previousPoint.y),
	}
	return [cp1, cp2]
}

type DrawLineChartOverlayOptions = {
	activeHoverRegions: Array<ActiveHoverRegion<LineChartHoverRegionInfo>>
	ctx: CanvasRenderingContext2D
	info: LineChartOverlayInfo
	overlayDiv: HTMLElement
}

export function drawLineChartOverlay(options: DrawLineChartOverlayOptions) {
	let {
		activeHoverRegions,
		ctx,
		info: { chartBox, xMax, xMin, yMax, yMin },
		overlayDiv,
	} = options
	let closestActiveHoverRegionForSeries = new Map<
		number,
		ActiveHoverRegion<LineChartHoverRegionInfo>
	>()
	for (let activeHoverRegion of activeHoverRegions) {
		let activeHoverRegionForSeries = closestActiveHoverRegionForSeries.get(
			activeHoverRegion.info.seriesIndex,
		)
		if (
			!activeHoverRegionForSeries ||
			activeHoverRegion.distance < activeHoverRegionForSeries.distance
		) {
			closestActiveHoverRegionForSeries.set(
				activeHoverRegion.info.seriesIndex,
				activeHoverRegion,
			)
		}
	}
	let closestActiveHoverRegions = Array.from(
		closestActiveHoverRegionForSeries.values(),
	)
	let tooltips: TooltipData[] = closestActiveHoverRegions.map(
		activeHoverRegion => {
			let x
			let label = activeHoverRegion.info.label
			if (label) {
				x = label
			} else {
				x = formatNumber(activeHoverRegion.info.point.x)
			}
			let y = formatNumber(activeHoverRegion.info.point.y)
			return {
				color: activeHoverRegion.info.color,
				text: `(${x}, ${y})`,
			}
		},
	)
	let closestActiveHoverRegion:
		| ActiveHoverRegion<LineChartHoverRegionInfo>
		| undefined
	for (let activeHoverRegion of closestActiveHoverRegions) {
		if (
			!closestActiveHoverRegion ||
			activeHoverRegion.distance < closestActiveHoverRegion.distance
		) {
			closestActiveHoverRegion = activeHoverRegion
		}
	}
	let tooltipOrigin = closestActiveHoverRegion
		? closestActiveHoverRegion.info.tooltipOriginPixels
		: undefined
	if (tooltipOrigin && tooltips.length === 1) {
		drawCrosshairs({
			chartBox,
			crosshairsColor: chartColors.current.crosshairsColor,
			ctx,
			origin: tooltipOrigin,
		})
	}
	if (tooltipOrigin) {
		drawTooltip({
			container: overlayDiv,
			origin: tooltipOrigin,
			values: tooltips,
		})
	}
	closestActiveHoverRegions.forEach(activeHoverRegion => {
		let point = activeHoverRegion.info.point
		drawPoint({
			chartBox,
			color: activeHoverRegion.info.color,
			ctx,
			point: { x: point.x, y: point.y },
			pointStyle: PointStyle.Circle,
			radius: chartConfig.pointRadius,
			xMax,
			xMin,
			yMax,
			yMin,
		})
		drawPoint({
			chartBox,
			color: '#00000022',
			ctx,
			point: { x: point.x, y: point.y },
			pointStyle: PointStyle.Circle,
			radius: chartConfig.pointRadius,
			xMax,
			xMin,
			yMax,
			yMin,
		})
	})
}

type DrawCrosshairsOptions = {
	chartBox: Box
	crosshairsColor: string
	ctx: CanvasRenderingContext2D
	origin: Point
}

export function drawCrosshairs(options: DrawCrosshairsOptions) {
	let { chartBox, crosshairsColor, ctx, origin } = options
	ctx.save()
	ctx.beginPath()
	ctx.setLineDash([4, 4])
	ctx.strokeStyle = crosshairsColor
	ctx.moveTo(origin.x, chartBox.y)
	ctx.lineTo(origin.x, chartBox.y + chartBox.h)
	ctx.moveTo(chartBox.x, origin.y)
	ctx.lineTo(chartBox.x + chartBox.w, origin.y)
	ctx.stroke()
	ctx.restore()
}

/** This function gets the location to draw on screen for a data point. */
function pointToPixels(options: {
	chartBox: Box
	point: Point
	xMax: number
	xMin: number
	yMax: number
	yMin: number
}): Point {
	let { chartBox, point, xMax, xMin, yMax, yMin } = options
	return {
		x:
			chartBox.x +
			(-xMin / (xMax - xMin)) * chartBox.w +
			(point.x / (xMax - xMin)) * chartBox.w,
		y:
			chartBox.y +
			chartBox.h -
			(-yMin / (yMax - yMin)) * chartBox.h -
			(point.y / (yMax - yMin)) * chartBox.h,
	}
}
