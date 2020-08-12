import { ActiveHoverRegion, HoverRegion, createChart } from './chart'
import {
	AxisLabelFormatter,
	Box,
	GridLineInterval,
	Point,
	RectCorner,
	computeBoxes,
	defaultAxisLabelFormatter,
	drawRoundedRect,
	drawXAxis,
	drawXAxisTitle,
	drawYAxisGridLines,
	drawYAxisLabels,
	drawYAxisTitle,
} from './common'
import { chartColors, chartConfig } from './config'
import { drawTooltip } from './tooltip'

export type DrawBarChartOptions = {
	data: BarChartData
	hideLegend?: boolean
	shouldDrawXAxisLabels?: boolean
	shouldDrawYAxisLabels?: boolean
	xAxisLabelFormatter?: AxisLabelFormatter
	xAxisTitle?: string
	yAxisGridLineInterval?: GridLineInterval
	yAxisLabelFormatter?: AxisLabelFormatter
	yAxisTitle?: string
	yMax?: number
	yMin?: number
}

export type BarChartData = BarChartSeries[]

export type BarChartSeries = {
	color: string
	data: BarChartPoint[]
	title: string
}

export type BarChartPoint = {
	x: number
	y: number | null
}

export type BarChartOverlayInfo = {
	chartBox: Box
	xAxisLabelFormatter: AxisLabelFormatter
	yAxisLabelFormatter: AxisLabelFormatter
}

export type BarChartHoverRegionInfo = {
	box: Box
	color: string
	point: Point
	tooltipOriginPixels: Point
}

export type DrawBarChartOutput = {
	hoverRegions: Array<HoverRegion<BarChartHoverRegionInfo>>
	overlayInfo: BarChartOverlayInfo
}

export function createBarChart(container: HTMLElement) {
	return createChart(container, drawBarChart, drawBarChartOverlay)
}

export function drawBarChart(
	ctx: CanvasRenderingContext2D,
	options: DrawBarChartOptions,
): DrawBarChartOutput {
	let {
		data,
		xAxisTitle,
		yAxisGridLineInterval: yAxisLineInterval,
		yAxisTitle,
	} = options
	let width = ctx.canvas.clientWidth
	let height = ctx.canvas.clientHeight
	let xAxisLabelFormatter =
		options.xAxisLabelFormatter ?? defaultAxisLabelFormatter
	let yAxisLabelFormatter =
		options.yAxisLabelFormatter ?? defaultAxisLabelFormatter
	let hoverRegions: Array<HoverRegion<BarChartHoverRegionInfo>> = []

	// compute bounds
	let yMin: number
	if (options.yMin !== undefined) {
		yMin = options.yMin
	} else {
		yMin = Math.min(
			0,
			...data.flatMap(series => series.data.map(p => p.y ?? Infinity)),
		)
	}
	let yMax: number
	if (options.yMax !== undefined) {
		yMax = options.yMax
	} else {
		yMax = Math.max(
			...data.flatMap(series => series.data.map(p => p.y ?? -Infinity)),
		)
	}
	if (!isFinite(yMax) || yMax === yMin) {
		yMax = yMin + 1
	}

	// compute boxes
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
		yAxisGridLineInterval: yAxisLineInterval,
		yAxisLabelFormatter,
		yMax,
		yMin,
	})

	let categories = data[0].data.map(({ x }) => xAxisLabelFormatter(x))

	let barGroupWidth =
		(chartBox.w - chartConfig.barGroupGap * (categories.length + 1)) /
		categories.length
	let barWidth =
		(barGroupWidth - chartConfig.barGap * (data.length - 1)) / data.length

	// draw x axis labels
	if (options.shouldDrawXAxisLabels ?? true) {
		drawBarChartXAxisLabels({
			barGroupGap: chartConfig.barGroupGap,
			box: xAxisLabelsBox,
			categories,
			ctx,
			groupWidth: barGroupWidth,
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

	// draw y axis labels
	if (options.shouldDrawYAxisLabels ?? true) {
		drawYAxisLabels({
			box: yAxisLabelsBox,
			ctx,
			fontSize: chartConfig.fontSize,
			gridLineInfo: yAxisGridLineInfo,
			height,
			yAxisLabelFormatter,
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

	// draw bars
	data.forEach((series, seriesIndex) => {
		series.data.forEach((point, pointIndex) => {
			if (point.y === null) {
				return
			}
			let box = {
				h: (point.y / (yMax - yMin)) * chartBox.h,
				w:
					(barGroupWidth - chartConfig.barGap * (data.length - 1)) /
					data.length,
				x:
					chartBox.x +
					(chartConfig.barGroupGap +
						(chartConfig.barGroupGap + barGroupWidth) * pointIndex) +
					(chartConfig.barGap + barWidth) * seriesIndex,
				y: chartBox.y + ((yMax - point.y) / (yMax - yMin)) * chartBox.h,
			}
			drawBar({
				box,
				chartBox,
				color: series.color + 'af',
				ctx,
				point,
			})
			let hoverRegion: HoverRegion<BarChartHoverRegionInfo> = {
				distance: (mouseX: number, _: number) => {
					return Math.abs(box.x + box.w / 2 - mouseX)
				},
				hitTest: (mouseX: number, mouseY: number) => {
					return (
						mouseX > Math.min(box.x, box.x + box.w) &&
						mouseX < Math.max(box.x, box.x + box.w) &&
						mouseY > chartBox.y &&
						mouseY < chartBox.y + chartBox.h
					)
				},
				info: {
					box,
					color: series.color,
					point: { x: point.x, y: point.y },
					tooltipOriginPixels: { x: box.x + box.w / 2, y: box.y },
				},
			}
			hoverRegions.push(hoverRegion)
			return { hoverRegions }
		})
	})

	let info: BarChartOverlayInfo = {
		chartBox,
		xAxisLabelFormatter,
		yAxisLabelFormatter,
	}

	return { hoverRegions, overlayInfo: info }
}

type DrawBarOptions = {
	box: Box
	chartBox: Box
	color: string
	ctx: CanvasRenderingContext2D
	point: BarChartPoint
}

function drawBar(options: DrawBarOptions) {
	let { box, color, ctx, point } = options
	let hoverRegions: Array<HoverRegion<BarChartHoverRegionInfo>> = []
	if (point.y == null) {
		return { hoverRegions }
	}
	let cornerMask =
		box.h > 0
			? RectCorner.TopLeft | RectCorner.TopRight
			: RectCorner.BottomLeft | RectCorner.BottomRight
	drawRoundedRect({
		box,
		cornerMask,
		ctx,
		fillColor: color,
		radius: Math.min(
			Math.abs(box.h / 2),
			Math.abs(box.w / 6),
			chartConfig.maxCornerRadius,
		),
		strokeColor: color,
		strokeWidth: chartConfig.barStrokeWidth,
	})
}

type DrawBarChartXAxisLabelsOptions = {
	barGroupGap: number
	box: Box
	categories: string[]
	ctx: CanvasRenderingContext2D
	groupWidth: number
	width: number
}

export function drawBarChartXAxisLabels(
	options: DrawBarChartXAxisLabelsOptions,
) {
	let { barGroupGap, box: box, categories, ctx, groupWidth, width } = options
	ctx.save()
	ctx.fillStyle = chartColors.current.labelColor
	ctx.textBaseline = 'bottom'
	ctx.textAlign = 'center'
	let labelStepSize = 1
	// find the smallest label step size at which labels do not overlap
	while (true) {
		let labelCenterSpacing = (barGroupGap + groupWidth) * labelStepSize
		let labelWidths = categories
			.filter((_, i) => (i + 1) % labelStepSize === 0)
			.map(category => ctx.measureText(category).width)
		let foundOverlap = false
		for (let i = 0; i < labelWidths.length - 2; i++) {
			let labelWidth = labelWidths[i]
			let nextLabelWidth = labelWidths[i + 1]
			if (labelWidth / 2 + nextLabelWidth / 2 > labelCenterSpacing) {
				foundOverlap = true
				break
			}
		}
		if (foundOverlap) {
			labelStepSize += 1
			continue
		} else {
			break
		}
	}
	categories.forEach((label, i) => {
		// render every labelStepSize label
		if ((i + 1) % labelStepSize !== 0) {
			return
		}
		let labelOffset =
			barGroupGap + groupWidth / 2 + (barGroupGap + groupWidth) * i
		// do not draw the label if it will overflow the chart
		if (
			box.x + labelOffset - ctx.measureText(label).width / 2 < 0 ||
			box.x + labelOffset + ctx.measureText(label).width / 2 > width
		) {
			return
		}
		ctx.fillText(label, box.x + labelOffset, box.y + box.h)
	})
	ctx.restore()
}

type DrawBarChartOverlayOptions = {
	activeHoverRegions: Array<ActiveHoverRegion<BarChartHoverRegionInfo>>
	ctx: CanvasRenderingContext2D
	info: BarChartOverlayInfo
}

export function drawBarChartOverlay(options: DrawBarChartOverlayOptions) {
	let {
		activeHoverRegions,
		ctx,
		info: { chartBox, xAxisLabelFormatter, yAxisLabelFormatter },
	} = options
	let activeHoverRegion = activeHoverRegions[0]
	if (activeHoverRegion) {
		let x = xAxisLabelFormatter(activeHoverRegion.info.point.x)
		let y = yAxisLabelFormatter(activeHoverRegion.info.point.y)
		let text = `(${x}, ${y})`
		let tooltip = {
			color: activeHoverRegion.info.color,
			text,
		}
		drawTooltip({
			centerHorizontal: true,
			chartBox,
			ctx,
			origin: activeHoverRegion.info.tooltipOriginPixels,
			values: [tooltip],
		})
		drawBar({
			box: activeHoverRegion.info.box,
			chartBox,
			color: '#00000022',
			ctx,
			point: activeHoverRegion.info.point,
		})
	}
}
