import {
	ActiveHoverRegion,
	DrawFunctionOptions,
	DrawFunctionOutput,
	HoverRegion,
	createChart,
} from "./chart"
import {
	Box,
	GridLineInterval,
	Point,
	RectCorner,
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
import { drawTooltip } from "./tooltip"

export type BarChartOptions = {
	groupGap?: number
	hideLegend?: boolean
	series: BarChartSeries[]
	shouldDrawXAxisLabels?: boolean
	shouldDrawYAxisLabels?: boolean
	xAxisTitle?: string
	yAxisGridLineInterval?: GridLineInterval
	yAxisTitle?: string
	yMax?: number
	yMin?: number
}

export type BarChartSeries = {
	color: string
	data: BarChartPoint[]
	title?: string
}

export type BarChartPoint = {
	label: string
	x: number
	y: number | null
}

export type BarChartOverlayInfo = {
	chartBox: Box
}

export type BarChartHoverRegionInfo = {
	box: Box
	color: string
	point: BarChartPoint
	pointLabel: string
	pointValue: number
	seriesTitle: string | undefined
	tooltipOriginPixels: Point
}

export function createBarChart(container: HTMLElement) {
	return createChart(container, drawBarChart, drawBarChartOverlay)
}

export function drawBarChart({
	ctx,
	options,
}: DrawFunctionOptions<BarChartOptions>): DrawFunctionOutput<
	BarChartOverlayInfo,
	BarChartHoverRegionInfo
> {
	let { series: data, xAxisTitle, yAxisGridLineInterval, yAxisTitle } = options
	let width = ctx.canvas.clientWidth
	let height = ctx.canvas.clientHeight
	let hoverRegions: Array<HoverRegion<BarChartHoverRegionInfo>> = []

	// Compute the bounds.
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
		yAxisGridLineInterval,
		yMax,
		yMin,
	})

	if (data[0] === undefined) throw Error()
	let categories = data[0].data.map(({ label }) => label)
	let groupGap = options.groupGap ?? chartConfig.barGroupGap
	let barGroupWidth =
		(chartBox.w - groupGap * (categories.length + 1)) / categories.length
	let barWidth =
		(barGroupWidth - chartConfig.barGap * (data.length - 1)) / data.length

	// Draw the X axis labels.
	if (options.shouldDrawXAxisLabels ?? true) {
		drawBarChartXAxisLabels({
			barGroupGap: groupGap,
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

	// Draw the bars.
	let hasMultipleSeries = data.length > 1
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
					(groupGap + (groupGap + barGroupWidth) * pointIndex) +
					(chartConfig.barGap + barWidth) * seriesIndex,
				y: chartBox.y + ((yMax - point.y) / (yMax - yMin)) * chartBox.h,
			}
			drawBar({
				box,
				color: series.color + "af",
				ctx,
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
					point,
					pointLabel: point.label,
					pointValue: point.y,
					seriesTitle: hasMultipleSeries ? series.title : undefined,
					tooltipOriginPixels: { x: box.x + box.w / 2, y: box.y },
				},
			}
			hoverRegions.push(hoverRegion)
		})
	})

	let info: BarChartOverlayInfo = {
		chartBox,
	}

	return { hoverRegions, overlayInfo: info }
}

type DrawBarOptions = {
	box: Box
	color: string
	ctx: CanvasRenderingContext2D
}

function drawBar(options: DrawBarOptions) {
	let { box, color, ctx } = options
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
	ctx.textBaseline = "bottom"
	ctx.textAlign = "center"
	let labelStepSize = 1
	// Find the smallest label step size at which labels do not overlap.
	while (true) {
		// This is how far the next label's center is.
		let nextLabelOffset = (barGroupGap + groupWidth) * labelStepSize
		let labelWidths = categories
			.filter((_, i) => (i + 1) % labelStepSize === 0)
			.map(category => ctx.measureText(category).width)
		let foundOverlap = false
		for (let i = 0; i < labelWidths.length - 1; i++) {
			let labelWidth = labelWidths[i]
			if (labelWidth === undefined) throw Error()
			let nextLabelWidth = labelWidths[i + 1]
			if (nextLabelWidth === undefined) throw Error()
			if (labelWidth / 2 + nextLabelWidth / 2 > nextLabelOffset) {
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
		// Render every `labelStepSize` label.
		if ((i + 1) % labelStepSize !== 0) {
			return
		}
		let labelOffset =
			barGroupGap + groupWidth / 2 + (barGroupGap + groupWidth) * i
		// Do not draw the label if it will overflow the chart.
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
	overlayDiv: HTMLElement
}

export function drawBarChartOverlay(options: DrawBarChartOverlayOptions) {
	let { activeHoverRegions, ctx, overlayDiv } = options
	let activeHoverRegion = activeHoverRegions[0]
	if (activeHoverRegion) {
		let seriesTitle = activeHoverRegion.info.seriesTitle
		let pointLabel = activeHoverRegion.info.pointLabel
		let pointValue = formatNumber(activeHoverRegion.info.pointValue)
		let text
		if (seriesTitle === undefined) {
			text = `(${pointLabel}, ${pointValue})`
		} else {
			text = `${seriesTitle} (${pointLabel}, ${pointValue})`
		}
		let tooltip = {
			color: activeHoverRegion.info.color,
			text,
		}
		drawTooltip({
			centerHorizontal: true,
			container: overlayDiv,
			labels: [tooltip],
			origin: activeHoverRegion.info.tooltipOriginPixels,
		})
		drawBar({
			box: activeHoverRegion.info.box,
			color: "#00000022",
			ctx,
		})
	}
}
