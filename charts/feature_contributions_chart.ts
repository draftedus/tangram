import {
	ActiveHoverRegion,
	DrawFunctionOptions,
	HoverRegion,
	createChart,
} from "./chart"
import {
	Box,
	Point,
	computeXAxisGridLineInfo,
	drawXAxisGridLines,
	drawXAxisLabels,
	drawXAxisTitle,
	drawYAxisTitle,
} from "./common"
import { chartColors, chartConfig } from "./config"
import { drawTooltip } from "./tooltip"

/** These are the options for displaying a feature contributions chart. */
export type FeatureContributionsChartOptions = {
	includeXAxisTitle?: boolean
	includeYAxisLabels?: boolean
	includeYAxisTitle?: boolean
	/** This is the color to fill the bars for negative values. You will probably want to use a shade of red. */
	negativeColor: string
	/** This is the color to fill the bars for positive values. You will probably want to use a shade of green. */
	positiveColor: string
	series: FeatureContributionsChartSeries[]
}

/** These are the configuration used across all feature contributions charts. */
export type FeatureContributionsChartConfig = {
	arrowDepth: number
	barGap: number
	seriesGap: number
	seriesWidth: number
}

export type FeatureContributionsChartSeries = {
	baseline: number
	baselineLabel: string
	label: string
	output: number
	outputLabel: string
	values: FeatureContributionsChartValue[]
}

export type FeatureContributionsChartValue = {
	feature: string
	value: number
}

export type FeatureContributionsChartHoverRegionInfo = {
	box: Box
	color: string
	direction: FeatureContributionsBoxDirection
	label: string
	tooltipOriginPixels: Point
}

export type FeatureContributionsChartOverlayInfo = {
	chartBox: Box
}

export type DrawFeatureContributionsChartOutput = {
	hoverRegions: Array<HoverRegion<FeatureContributionsChartHoverRegionInfo>>
	overlayInfo: FeatureContributionsChartOverlayInfo
}

export function createFeatureContributionsChart(container: HTMLElement) {
	return createChart(
		container,
		drawFeatureContributionsChart,
		drawFeatureContributionsChartOverlay,
	)
}

export function drawFeatureContributionsChart({
	ctx,
	options,
}: DrawFunctionOptions<
	FeatureContributionsChartOptions
>): DrawFeatureContributionsChartOutput {
	let {
		includeXAxisTitle,
		includeYAxisLabels,
		includeYAxisTitle,
		negativeColor,
		positiveColor,
		series: data,
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
	let hoverRegions: Array<HoverRegion<
		FeatureContributionsChartHoverRegionInfo
	>> = []

	// Compute the bounds.
	let xMin = Math.min(
		...data.flatMap(
			classFeatureContributionValues => classFeatureContributionValues.baseline,
		),
		...data.flatMap(
			classFeatureContributionValues => classFeatureContributionValues.output,
		),
	)
	let xMax = Math.max(
		...data.flatMap(
			classFeatureContributionValues =>
				classFeatureContributionValues.baseline +
				classFeatureContributionValues.values
					.filter(v => v.value > 0)
					.reduce(
						(positiveValuesSum, { value }) => (positiveValuesSum += value),
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
		drawXAxisTitle({ box: topXAxisTitleBox, ctx, title: "Contributions" })
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
		drawYAxisTitle({ box: yAxisTitlesBox, ctx, title: "Class" })
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

	let categories = data.map(({ label }) => label)
	if (includeYAxisLabels) {
		drawFeatureContributionsChartYAxisLabels({
			box: yAxisLabelsBox,
			categories,
			ctx,
			width,
		})
	}

	// Draw the series separators.
	for (let i = 1; i < categories.length; i++) {
		let y =
			chartBox.y +
			i * chartConfig.featureContributionsSeriesHeight +
			(i - 1) * chartConfig.featureContributionsSeriesGap +
			chartConfig.featureContributionsSeriesGap / 2
		ctx.save()
		ctx.strokeStyle = chartColors.current.gridLineColor
		ctx.moveTo(chartBox.x, y)
		ctx.lineTo(chartBox.x + chartBox.w, y)
		ctx.stroke()
		ctx.restore()
	}

	let valueWidthMultiplier = chartBox.w / (xMax - xMin)
	data.forEach((series, seriesIndex) => {
		let sumPositives = series.values
			.filter(({ value }) => value > 0)
			.reduce((posSum, { value }) => (posSum += value), 0.0)
		let min = Math.min(series.baseline, series.output)
		let max = series.baseline + sumPositives
		let width = max - min
		let boxHeight =
			(chartConfig.featureContributionsSeriesHeight -
				chartConfig.featureContributionsBarGap) /
			2
		let box = {
			h: chartConfig.featureContributionsSeriesHeight,
			w: width * valueWidthMultiplier,
			x: chartBox.x + (min - xMin) * valueWidthMultiplier,
			y:
				chartBox.y +
				(chartConfig.featureContributionsSeriesGap +
					chartConfig.featureContributionsSeriesHeight) *
					seriesIndex,
		}
		let output = drawFeatureContributionsSeries({
			box,
			boxHeight,
			ctx,
			negativeColor,
			positiveColor,
			series,
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

type DrawFeatureContributionSeriesOptions = {
	box: Box
	boxHeight: number
	ctx: CanvasRenderingContext2D
	negativeColor: string
	positiveColor: string
	series: FeatureContributionsChartSeries
	valueWidthMultiplier: number
}

type DrawFeatureContributionsSeriesOutput = {
	hoverRegions: Array<HoverRegion<FeatureContributionsChartHoverRegionInfo>>
}

function drawFeatureContributionsSeries(
	options: DrawFeatureContributionSeriesOptions,
): DrawFeatureContributionsSeriesOutput {
	let hoverRegions: Array<HoverRegion<
		FeatureContributionsChartHoverRegionInfo
	>> = []
	let {
		box,
		boxHeight,
		ctx,
		negativeColor,
		positiveColor,
		series,
		valueWidthMultiplier,
	} = options

	let min = Math.min(series.baseline, series.output)

	// Draw the positive boxes which start at the baseline and go to the max, ending with the remaining features box.
	let positiveValues = series.values
		.filter(({ value }) => value > 0)
		.sort((a, b) => (a.value > b.value ? -1 : 1))
	let x = box.x + (series.baseline - min) * valueWidthMultiplier
	let positiveValuesIndex = 0
	// Draw the baseline value and label.
	ctx.textBaseline = "bottom"
	ctx.textAlign = "right"
	ctx.fillText(`baseline`, x - chartConfig.labelPadding, box.y + boxHeight / 2)
	ctx.textBaseline = "top"
	ctx.textAlign = "right"
	ctx.fillText(
		series.baselineLabel,
		x - chartConfig.labelPadding,
		box.y + boxHeight / 2,
	)
	while (positiveValuesIndex < positiveValues.length) {
		let featureContributionValue = positiveValues[positiveValuesIndex]
		if (featureContributionValue === undefined) throw Error()
		let width = featureContributionValue.value * valueWidthMultiplier
		if (width < chartConfig.featureContributionsArrowDepth * 2) {
			break
		}
		let valueBox = {
			h: boxHeight,
			w: width,
			x,
			y: box.y,
		}
		drawFeatureContributionBox({
			box: valueBox,
			color: positiveColor,
			ctx,
			direction: FeatureContributionsBoxDirection.Negative,
			label: `${featureContributionValue.feature}`,
		})
		hoverRegions.push(
			featureContributionsChartHoverRegion({
				box: valueBox,
				color: positiveColor,
				direction: FeatureContributionsBoxDirection.Negative,
				label: `${featureContributionValue.feature}`,
				tooltipOriginPixels: {
					...valueBox,
					x: valueBox.x + valueBox.w / 2,
				},
			}),
		)
		x += width
		positiveValuesIndex += 1
	}
	let nRemainingFeatures = 0
	let remainingFeaturesBoxWidth = 0
	for (let i = positiveValuesIndex; i < positiveValues.length; i++) {
		let featureContributionValue = positiveValues[i]
		if (featureContributionValue === undefined) throw Error()
		let width = featureContributionValue.value * valueWidthMultiplier
		remainingFeaturesBoxWidth += width
		nRemainingFeatures += 1
	}
	if (remainingFeaturesBoxWidth > 0) {
		let remainingFeaturesBox = {
			h: boxHeight,
			w: remainingFeaturesBoxWidth,
			x,
			y: box.y,
		}
		drawFeatureContributionBox({
			box: remainingFeaturesBox,
			color: `${positiveColor}33`,
			ctx,
			direction: FeatureContributionsBoxDirection.Negative,
			label: `${nRemainingFeatures} other features`,
		})
		hoverRegions.push(
			featureContributionsChartHoverRegion({
				box: remainingFeaturesBox,
				color: `${positiveColor}33`,
				direction: FeatureContributionsBoxDirection.Negative,
				label: `${nRemainingFeatures} other features`,
				tooltipOriginPixels: {
					...remainingFeaturesBox,
					x: remainingFeaturesBox.x + remainingFeaturesBox.w / 2,
				},
			}),
		)
	}

	// Draw the negative boxes which start at the max and go to the output, starting with the remaining features box.
	x = box.x + box.w
	let y = box.y + boxHeight + chartConfig.featureContributionsBarGap
	let negativeValues = series.values
		.filter(({ value }) => value < 0)
		.sort((a, b) => (a.value > b.value ? -1 : 1))
	remainingFeaturesBoxWidth = 0
	nRemainingFeatures = 0
	let negativeValuesIndex = 0
	while (negativeValuesIndex < negativeValues.length) {
		let featureContributionValue = negativeValues[negativeValuesIndex]
		if (featureContributionValue === undefined) throw Error()
		let width = featureContributionValue.value * valueWidthMultiplier
		if (width < -chartConfig.featureContributionsArrowDepth * 2) {
			break
		}
		remainingFeaturesBoxWidth += width
		nRemainingFeatures += 1
		negativeValuesIndex += 1
	}
	if (remainingFeaturesBoxWidth < 0) {
		let remainingFeaturesBox = {
			h: boxHeight,
			w: remainingFeaturesBoxWidth,
			x,
			y,
		}
		x += remainingFeaturesBoxWidth
		drawFeatureContributionBox({
			box: remainingFeaturesBox,
			color: `${negativeColor}33`,
			ctx,
			direction: FeatureContributionsBoxDirection.Positive,
			label: `${nRemainingFeatures} other features`,
		})
		hoverRegions.push(
			featureContributionsChartHoverRegion({
				box: remainingFeaturesBox,
				color: `${negativeColor}33`,
				direction: FeatureContributionsBoxDirection.Positive,
				label: `${nRemainingFeatures} other features`,
				tooltipOriginPixels: {
					...remainingFeaturesBox,
					x: remainingFeaturesBox.x + remainingFeaturesBox.w / 2,
				},
			}),
		)
	}
	for (let i = negativeValuesIndex; i < negativeValues.length; i++) {
		let featureContributionValue = negativeValues[i]
		if (featureContributionValue === undefined) throw Error()
		let width = featureContributionValue.value * valueWidthMultiplier
		let valueBox = {
			h: boxHeight,
			w: width,
			x,
			y,
		}
		drawFeatureContributionBox({
			box: valueBox,
			color: negativeColor,
			ctx,
			direction: FeatureContributionsBoxDirection.Positive,
			label: `${featureContributionValue.feature}`,
		})
		hoverRegions.push(
			featureContributionsChartHoverRegion({
				box: valueBox,
				color: negativeColor,
				direction: FeatureContributionsBoxDirection.Positive,
				label: `${featureContributionValue.feature}`,
				tooltipOriginPixels: {
					...valueBox,
					x: valueBox.x + valueBox.w / 2,
				},
			}),
		)
		x += width
	}
	// Draw the output value and label.
	ctx.textBaseline = "bottom"
	ctx.fillText(
		`output`,
		x - chartConfig.labelPadding,
		box.y + boxHeight + chartConfig.featureContributionsBarGap + boxHeight / 2,
	)
	ctx.textBaseline = "top"
	ctx.fillText(
		series.outputLabel,
		x - chartConfig.labelPadding,
		box.y + boxHeight + chartConfig.featureContributionsBarGap + boxHeight / 2,
	)

	return {
		hoverRegions,
	}
}

type DrawFeatureContributionsChartYAxisLabelsOptions = {
	box: Box
	categories: string[]
	ctx: CanvasRenderingContext2D
	width: number
}

function drawFeatureContributionsChartYAxisLabels(
	options: DrawFeatureContributionsChartYAxisLabelsOptions,
) {
	let { box, categories, ctx } = options
	ctx.textAlign = "end"
	categories.forEach((label, i) => {
		let labelOffset =
			chartConfig.featureContributionsSeriesHeight / 2 +
			(chartConfig.featureContributionsSeriesGap +
				chartConfig.featureContributionsSeriesHeight) *
				i
		ctx.textBaseline = "middle"
		ctx.fillText(label, box.x + box.w, box.y + labelOffset)
	})
}

type DrawFeatureContributionsChartOverlayOptions = {
	activeHoverRegions: Array<
		ActiveHoverRegion<FeatureContributionsChartHoverRegionInfo>
	>
	ctx: CanvasRenderingContext2D
	info: FeatureContributionsChartOverlayInfo
	overlayDiv: HTMLElement
}

export function drawFeatureContributionsChartOverlay(
	options: DrawFeatureContributionsChartOverlayOptions,
) {
	let { activeHoverRegions, ctx, info, overlayDiv } = options
	drawFeatureContributionTooltips({
		activeHoverRegions,
		chartBox: info.chartBox,
		ctx,
		overlayDiv,
	})
	activeHoverRegions.forEach(activeHoverRegion => {
		drawFeatureContributionBox({
			box: activeHoverRegion.info.box,
			color: "#00000022",
			ctx,
			direction: activeHoverRegion.info.direction,
			label: "",
		})
	})
}

type DrawFeatureContributionTooltipOptions = {
	activeHoverRegions: Array<
		ActiveHoverRegion<FeatureContributionsChartHoverRegionInfo>
	>
	chartBox: Box
	ctx: CanvasRenderingContext2D
	overlayDiv: HTMLElement
}

let drawFeatureContributionTooltips = (
	options: DrawFeatureContributionTooltipOptions,
) => {
	let { activeHoverRegions, overlayDiv } = options
	for (let activeHoverRegion of activeHoverRegions) {
		drawTooltip({
			centerHorizontal: true,
			container: overlayDiv,
			labels: [
				{
					color: activeHoverRegion.info.color,
					text: activeHoverRegion.info.label,
				},
			],
			origin: activeHoverRegion.info.tooltipOriginPixels,
		})
	}
}

type RegisterFeatureContributionsChartHoverRegionOptions = {
	box: Box
	color: string
	direction: FeatureContributionsBoxDirection
	label: string
	tooltipOriginPixels: Box
}

let featureContributionsChartHoverRegion = (
	options: RegisterFeatureContributionsChartHoverRegionOptions,
): HoverRegion<FeatureContributionsChartHoverRegionInfo> => {
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
		} as FeatureContributionsChartHoverRegionInfo,
	}
}

type DrawFeatureContributionBoxOptions = {
	box: Box
	color: string
	ctx: CanvasRenderingContext2D
	direction: FeatureContributionsBoxDirection
	label: string
}

enum FeatureContributionsBoxDirection {
	Positive,
	Negative,
}

export let drawFeatureContributionBox = (
	options: DrawFeatureContributionBoxOptions,
) => {
	let { box, color, ctx, direction, label } = options

	let textPadding = 4
	let arrowDepth =
		direction == FeatureContributionsBoxDirection.Negative
			? chartConfig.featureContributionsArrowDepth
			: -chartConfig.featureContributionsArrowDepth

	ctx.save()
	if (color) {
		ctx.strokeStyle = color
		ctx.fillStyle = color
	}

	ctx.lineWidth = 1
	ctx.lineCap = "butt"

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
	ctx.textBaseline = "middle"
	ctx.textAlign = "center"
	ctx.fillStyle = "#fff"

	if (
		labelWidth <=
		Math.abs(box.w) -
			textPadding -
			chartConfig.featureContributionsArrowDepth * 2
	) {
		ctx.fillText(label, box.x + (box.w + arrowDepth) / 2, box.y + box.h / 2)
	}

	ctx.restore()
}
