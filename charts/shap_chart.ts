import { ActiveHoverRegion, HoverRegion, createChart } from './chart'
import {
	Box,
	Point,
	computeXAxisGridLineInfo,
	drawXAxisGridLines,
	drawXAxisLabels,
	drawXAxisTitle,
	drawYAxisTitle,
} from './common'
import { chartColors, chartConfig } from './config'
import { drawTooltip } from './tooltip'

/** These are the options for displaying a SHAP chart. */
export type ShapChartOptions = {
	/** The data to display in the chart. */
	data: ShapChartData
	/** The default is true. */
	includeXAxisTitle?: boolean
	/** The default is true. */
	includeYAxisLabels?: boolean
	/** The default is true. */
	includeYAxisTitle?: boolean
	/** This is the color to fill the bars for negative values. You will probably want to use a shade of red. */
	negativeColor: string
	/** This is the color to fill the bars for positive values. You will probably want to use a shade of green. */
	positiveColor: string
}

/** These are the configuration used across all SHAP charts. */
export type ShapChartConfig = {
	arrowDepth: number
	barGap: number
	groupGap: number
	groupWidth: number
}

export type ShapChartData = ShapChartSeries[]

export type ShapChartSeries = {
	baseline: number
	baselineLabel: string
	label: string
	output: number
	outputLabel: string
	values: ShapValue[]
}

export type ShapValue = {
	feature: string
	value: number
}

export type ShapChartHoverRegionInfo = {
	box: Box
	color: string
	direction: Direction
	label: string
	tooltipOriginPixels: Point
}

export type ShapChartOverlayInfo = {
	chartBox: Box
}

export type DrawMultiShapChartOutput = {
	hoverRegions: Array<HoverRegion<ShapChartHoverRegionInfo>>
	overlayInfo: ShapChartOverlayInfo
}

export function createShapChart(container: HTMLElement) {
	return createChart(container, drawShapChart, drawShapChartOverlay)
}

export function drawShapChart(
	ctx: CanvasRenderingContext2D,
	options: ShapChartOptions,
): DrawMultiShapChartOutput {
	let {
		data,
		includeXAxisTitle,
		includeYAxisLabels,
		includeYAxisTitle,
		negativeColor,
		positiveColor,
	} = options

	let height = ctx.canvas.clientHeight
	let width = ctx.canvas.clientWidth
	let {
		bottomPadding,
		fontSize,
		labelPadding,
		leftPadding,
		rightPadding,
		topPadding,
	} = chartConfig

	let annotationsPadding = 80
	let hoverRegions: Array<HoverRegion<ShapChartHoverRegionInfo>> = []

	// Compute the bounds.
	let xMin = Math.min(
		...data.flatMap(classShapValues => classShapValues.baseline),
		...data.flatMap(classShapValues => classShapValues.output),
	)
	let xMax = Math.max(
		...data.flatMap(
			classShapValues =>
				classShapValues.baseline +
				classShapValues.values
					.filter(v => v.value > 0)
					.reduce(
						(positiveValuesSum, shapValue) =>
							(positiveValuesSum += shapValue.value),
						0.0,
					),
		),
	)

	let yAxisLabelsWidth = Math.max(
		...data.map(value => ctx.measureText(value.label).width),
	)
	let chartWidth =
		width -
		(leftPadding +
			(includeYAxisLabels ? yAxisLabelsWidth + labelPadding : 0) +
			rightPadding +
			(includeYAxisTitle ? fontSize + labelPadding : 0) +
			annotationsPadding)

	let chartHeight =
		height -
		(topPadding +
			fontSize +
			labelPadding +
			fontSize +
			labelPadding +
			(includeXAxisTitle ? labelPadding + fontSize : 0) +
			bottomPadding)

	let chartBox = {
		h: chartHeight,
		w: chartWidth,
		x:
			leftPadding +
			(includeYAxisTitle ? labelPadding + fontSize : 0) +
			(includeYAxisLabels ? yAxisLabelsWidth + labelPadding : 0) +
			annotationsPadding,
		y:
			topPadding +
			(includeXAxisTitle ? labelPadding + fontSize : 0) +
			fontSize +
			labelPadding,
	}

	let topXAxisTitleBox = {
		h: fontSize,
		w: chartWidth,
		x:
			leftPadding +
			(includeYAxisTitle ? labelPadding + fontSize : 0) +
			(includeYAxisLabels ? yAxisLabelsWidth + labelPadding : 0) +
			annotationsPadding,
		y: topPadding,
	}
	if (includeXAxisTitle) {
		drawXAxisTitle({ box: topXAxisTitleBox, ctx, title: 'Contributions' })
	}

	let topXAxisLabelsBox = {
		h: fontSize,
		w: chartWidth,
		x:
			leftPadding +
			(includeYAxisTitle ? labelPadding + fontSize : 0) +
			(includeYAxisLabels ? yAxisLabelsWidth + labelPadding : 0) +
			annotationsPadding,
		y: topPadding + (includeXAxisTitle ? labelPadding + fontSize : 0),
	}

	let bottomXAxisLabelsBox = {
		h: fontSize,
		w: chartWidth,
		x:
			leftPadding +
			(includeYAxisTitle ? labelPadding + fontSize : 0) +
			(includeYAxisLabels ? yAxisLabelsWidth + labelPadding : 0) +
			annotationsPadding,
		y:
			topPadding +
			fontSize +
			labelPadding +
			(includeXAxisTitle ? labelPadding + fontSize : 0) +
			chartHeight +
			labelPadding,
	}

	let yAxisLabelsBox = {
		h: chartHeight,
		w: yAxisLabelsWidth,
		x: leftPadding + (includeYAxisTitle ? labelPadding + fontSize : 0),
		y:
			topPadding +
			(includeXAxisTitle ? labelPadding + fontSize : 0) +
			fontSize +
			labelPadding,
	}

	let yAxisTitlesBox = {
		h: chartHeight,
		w: fontSize,
		x: leftPadding,
		y:
			topPadding +
			(includeXAxisTitle ? labelPadding + fontSize : 0) +
			fontSize +
			labelPadding,
	}

	if (includeYAxisTitle) {
		drawYAxisTitle({ box: yAxisTitlesBox, ctx, title: 'Class' })
	}

	// Compute the grid line info.
	let xAxisGridLineInfo = computeXAxisGridLineInfo({
		chartWidth: chartBox.w,
		ctx,
		xMax,
		xMin,
	})

	drawXAxisGridLines({
		box: chartBox,
		ctx,
		xAxisGridLineInfo,
	})

	drawXAxisLabels({
		box: topXAxisLabelsBox,
		ctx,
		gridLineInfo: xAxisGridLineInfo,
		width,
	})

	drawXAxisLabels({
		box: bottomXAxisLabelsBox,
		ctx,
		gridLineInfo: xAxisGridLineInfo,
		width,
	})

	let categories = data.map(classShapValues => classShapValues.label)
	if (includeYAxisLabels) {
		drawShapChartYAxisLabels({
			box: yAxisLabelsBox,
			categories,
			ctx,
			width,
		})
	}

	// Draw the group separators.
	for (let i = 1; i < categories.length; i++) {
		let y =
			chartBox.y +
			i * chartConfig.shapGroupHeight +
			(i - 1) * chartConfig.shapGroupGap +
			chartConfig.shapGroupGap / 2
		ctx.save()
		ctx.strokeStyle = chartColors.current.gridLineColor
		ctx.moveTo(chartBox.x, y)
		ctx.lineTo(chartBox.x + chartBox.w, y)
		ctx.stroke()
		ctx.restore()
	}

	let valueWidthMultiplier = chartBox.w / (xMax - xMin)
	data.forEach((shapValues, shapValuesIndex) => {
		// Draw the shap boxes.
		let sumPositives = shapValues.values
			.filter(shapValue => shapValue.value > 0)
			.reduce((posSum, shapValue) => (posSum += shapValue.value), 0.0)
		let min = Math.min(shapValues.baseline, shapValues.output)
		let max = shapValues.baseline + sumPositives
		let width = max - min
		let shapValueBoxHeight =
			(chartConfig.shapGroupHeight - chartConfig.shapBarGap) / 2

		let box = {
			h: chartConfig.shapGroupHeight,
			w: width * valueWidthMultiplier,
			x: chartBox.x + (min - xMin) * valueWidthMultiplier,
			y:
				chartBox.y +
				(chartConfig.shapGroupGap + chartConfig.shapGroupHeight) *
					shapValuesIndex,
		}

		let output = drawShap({
			box,
			ctx,
			negativeColor,
			positiveColor,
			shapValueBoxHeight,
			shapValues,
			valueWidthMultiplier,
		})
		hoverRegions.push(...output.hoverRegions)
	})

	return {
		hoverRegions,
		overlayInfo: {
			chartBox,
		},
	}
}

type DrawShapOptions = {
	box: Box
	ctx: CanvasRenderingContext2D
	negativeColor: string
	positiveColor: string
	shapValueBoxHeight: number
	shapValues: ShapChartSeries
	valueWidthMultiplier: number
}

type DrawShapChartOutput = {
	hoverRegions: Array<HoverRegion<ShapChartHoverRegionInfo>>
}

function drawShap(options: DrawShapOptions): DrawShapChartOutput {
	let hoverRegions: Array<HoverRegion<ShapChartHoverRegionInfo>> = []
	let {
		box,
		ctx,
		negativeColor,
		positiveColor,
		shapValueBoxHeight,
		shapValues,
		valueWidthMultiplier,
	} = options

	let min = Math.min(shapValues.baseline, shapValues.output)

	// Draw the positive box which starts at baseline and goes to max.
	let positiveValues = shapValues.values
		.filter(shapValue => shapValue.value > 0)
		.sort((a, b) => (a.value > b.value ? -1 : 1))
	let x = box.x + (shapValues.baseline - min) * valueWidthMultiplier
	let positiveValuesIndex = 0
	ctx.textBaseline = 'bottom'
	ctx.textAlign = 'right'
	ctx.fillText(
		`baseline`,
		x - chartConfig.labelPadding,
		box.y + shapValueBoxHeight / 2,
	)
	ctx.textBaseline = 'top'
	ctx.textAlign = 'right'
	ctx.fillText(
		shapValues.baselineLabel,
		x - chartConfig.labelPadding,
		box.y + shapValueBoxHeight / 2,
	)
	while (positiveValuesIndex < positiveValues.length) {
		let shapValue = positiveValues[positiveValuesIndex]
		let width = shapValue.value * valueWidthMultiplier
		if (width < chartConfig.shapArrowDepth * 2) {
			break
		}
		let valueBox = {
			h: shapValueBoxHeight,
			w: width,
			x,
			y: box.y,
		}
		drawShapBox({
			box: valueBox,
			color: positiveColor,
			ctx,
			direction: Direction.Right,
			label: `${shapValue.feature}`,
		})
		hoverRegions.push(
			shapChartHoverRegion({
				box: valueBox,
				color: positiveColor,
				direction: Direction.Right,
				label: `${shapValue.feature}`,
				tooltipOriginPixels: {
					...valueBox,
					x: valueBox.x + valueBox.w / 2,
				},
			}),
		)
		x += width
		positiveValuesIndex += 1
	}
	// The rest of the values should be grouped into a single group.
	let nGrouped = 0
	let groupedBoxWidth = 0
	for (let i = positiveValuesIndex; i < positiveValues.length; i++) {
		let shapValue = positiveValues[i]
		let width = shapValue.value * valueWidthMultiplier
		groupedBoxWidth += width
		nGrouped += 1
	}
	if (groupedBoxWidth > 0) {
		let groupedValueBox = {
			h: shapValueBoxHeight,
			w: groupedBoxWidth,
			x,
			y: box.y,
		}
		drawShapBox({
			box: groupedValueBox,
			color: `${positiveColor}33`,
			ctx,
			direction: Direction.Right,
			label: `${nGrouped} other features`,
		})
		hoverRegions.push(
			shapChartHoverRegion({
				box: groupedValueBox,
				color: `${positiveColor}33`,
				direction: Direction.Right,
				label: `${nGrouped} other features`,
				tooltipOriginPixels: {
					...groupedValueBox,
					x: groupedValueBox.x + groupedValueBox.w / 2,
				},
			}),
		)
	}

	// Draw the negative box.
	x = box.x + box.w
	let y = box.y + shapValueBoxHeight + chartConfig.shapBarGap
	let negativeValues = shapValues.values
		.filter(shapValue => shapValue.value < 0)
		.sort((a, b) => (a.value > b.value ? -1 : 1))
	groupedBoxWidth = 0
	nGrouped = 0
	// The first values should be grouped together.
	let negativeValuesIndex = 0
	while (negativeValuesIndex < negativeValues.length) {
		let shapValue = negativeValues[negativeValuesIndex]
		let width = shapValue.value * valueWidthMultiplier
		if (width < -chartConfig.shapArrowDepth * 2) {
			break
		}
		groupedBoxWidth += width
		nGrouped += 1
		negativeValuesIndex += 1
	}
	if (groupedBoxWidth < 0) {
		let groupedValueBox = {
			h: shapValueBoxHeight,
			w: groupedBoxWidth,
			x,
			y,
		}
		x += groupedBoxWidth
		drawShapBox({
			box: groupedValueBox,
			color: `${negativeColor}33`,
			ctx,
			direction: Direction.Left,
			label: `${nGrouped} other features`,
		})
		hoverRegions.push(
			shapChartHoverRegion({
				box: groupedValueBox,
				color: `${negativeColor}33`,
				direction: Direction.Left,
				label: `${nGrouped} other features`,
				tooltipOriginPixels: {
					...groupedValueBox,
					x: groupedValueBox.x + groupedValueBox.w / 2,
				},
			}),
		)
	}
	for (let i = negativeValuesIndex; i < negativeValues.length; i++) {
		let shapValue = negativeValues[i]
		let width = shapValue.value * valueWidthMultiplier
		let valueBox = {
			h: shapValueBoxHeight,
			w: width,
			x,
			y,
		}
		drawShapBox({
			box: valueBox,
			color: negativeColor,
			ctx,
			direction: Direction.Left,
			label: `${shapValue.feature}`,
		})
		hoverRegions.push(
			shapChartHoverRegion({
				box: valueBox,
				color: negativeColor,
				direction: Direction.Left,
				label: `${shapValue.feature}`,
				tooltipOriginPixels: {
					...valueBox,
					x: valueBox.x + valueBox.w / 2,
				},
			}),
		)
		x += width
	}

	ctx.textBaseline = 'bottom'
	ctx.fillText(
		`output`,
		x - chartConfig.labelPadding,
		box.y +
			shapValueBoxHeight +
			chartConfig.shapBarGap +
			shapValueBoxHeight / 2,
	)
	ctx.textBaseline = 'top'
	ctx.fillText(
		shapValues.outputLabel,
		x - chartConfig.labelPadding,
		box.y +
			shapValueBoxHeight +
			chartConfig.shapBarGap +
			shapValueBoxHeight / 2,
	)

	return {
		hoverRegions,
	}
}

type DrawShapChartYAxisLabelsOptions = {
	box: Box
	categories: string[]
	ctx: CanvasRenderingContext2D
	width: number
}

function drawShapChartYAxisLabels(options: DrawShapChartYAxisLabelsOptions) {
	let { box, categories, ctx } = options
	ctx.textAlign = 'end'
	categories.forEach((label, i) => {
		let labelOffset =
			chartConfig.shapGroupHeight / 2 +
			(chartConfig.shapGroupGap + chartConfig.shapGroupHeight) * i
		ctx.textBaseline = 'middle'
		ctx.fillText(label, box.x + box.w, box.y + labelOffset)
	})
}

type DrawShapChartOverlayOptions = {
	activeHoverRegions: Array<ActiveHoverRegion<ShapChartHoverRegionInfo>>
	ctx: CanvasRenderingContext2D
	info: ShapChartOverlayInfo
	overlayDiv: HTMLElement
}

export function drawShapChartOverlay(options: DrawShapChartOverlayOptions) {
	let { activeHoverRegions, ctx, info, overlayDiv } = options
	drawShapTooltips({
		activeHoverRegions,
		chartBox: info.chartBox,
		ctx,
		overlayDiv,
	})
	activeHoverRegions.forEach(activeHoverRegion => {
		drawShapBox({
			box: activeHoverRegion.info.box,
			color: '#00000022',
			ctx,
			direction: activeHoverRegion.info.direction,
			label: '',
		})
	})
}

type DrawShapTooltipOptions = {
	activeHoverRegions: Array<ActiveHoverRegion<ShapChartHoverRegionInfo>>
	chartBox: Box
	ctx: CanvasRenderingContext2D
	overlayDiv: HTMLElement
}

let drawShapTooltips = (options: DrawShapTooltipOptions) => {
	let { activeHoverRegions, overlayDiv } = options
	for (let activeHoverRegion of activeHoverRegions) {
		drawTooltip({
			centerHorizontal: true,
			container: overlayDiv,
			origin: activeHoverRegion.info.tooltipOriginPixels,
			values: [
				{
					color: activeHoverRegion.info.color,
					text: activeHoverRegion.info.label,
				},
			],
		})
	}
}

type RegisterShapChartHoverRegionOptions = {
	box: Box
	color: string
	direction: Direction
	label: string
	tooltipOriginPixels: Box
}

let shapChartHoverRegion = (
	options: RegisterShapChartHoverRegionOptions,
): HoverRegion<ShapChartHoverRegionInfo> => {
	let { box, color, direction, label, tooltipOriginPixels } = options
	return {
		distance: (mouseX: number, mouseY: number) => {
			return (box.x - mouseX) ** 2 + (box.y - mouseY) ** 2
		},
		hitTest: (mouseX: number, mouseY: number) => {
			return (
				mouseX > Math.min(box.x, box.x + box.w) &&
				mouseX < Math.max(box.x, box.x + box.w) &&
				mouseY > box.y &&
				mouseY < box.y + box.h
			)
		},
		info: {
			box,
			color,
			direction,
			label,
			tooltipOriginPixels,
		} as ShapChartHoverRegionInfo,
	}
}

type DrawShapBoxOptions = {
	box: Box
	color: string
	ctx: CanvasRenderingContext2D
	direction: Direction
	label: string
}

enum Direction {
	Left,
	Right,
	Up,
	Down,
}

export let drawShapBox = (options: DrawShapBoxOptions) => {
	let { box, color, ctx, direction, label } = options

	let textPadding = 4
	let arrowDepth =
		direction == Direction.Right
			? chartConfig.shapArrowDepth
			: -chartConfig.shapArrowDepth

	ctx.save()
	if (color) {
		ctx.strokeStyle = color
		ctx.fillStyle = color
	}

	ctx.lineWidth = 1
	ctx.lineCap = 'butt'

	let width = box.w
	ctx.beginPath()
	ctx.moveTo(box.x, box.y)
	let drawEndArrow = true
	let drawStartArrow = true

	// Draw the endpoint.
	if (drawEndArrow) {
		ctx.lineTo(box.x + width - arrowDepth, box.y)
		ctx.lineTo(box.x + width, box.y + box.h / 2)
		ctx.lineTo(box.x + width - arrowDepth, box.y + box.h)
	} else {
		ctx.lineTo(box.x + width, box.y)
		ctx.lineTo(box.x + width, box.y + box.h)
	}

	// Draw the startpoint.
	if (drawStartArrow) {
		ctx.lineTo(box.x, box.y + box.h)
		ctx.lineTo(box.x + arrowDepth, box.y + box.h / 2)
		ctx.lineTo(box.x, box.y)
	} else {
		ctx.lineTo(box.x, box.y + box.h)
		ctx.lineTo(box.x, box.y)
	}

	ctx.fill()

	let labelWidth = ctx.measureText(label).width
	ctx.textBaseline = 'middle'
	ctx.textAlign = 'center'
	ctx.fillStyle = '#fff'

	if (
		labelWidth <=
		Math.abs(box.w) - textPadding - chartConfig.shapArrowDepth * 2
	) {
		ctx.fillText(label, box.x + (box.w + arrowDepth) / 2, box.y + box.h / 2)
	}

	ctx.restore()
}
