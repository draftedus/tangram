import { chartColors, chartConfig } from "./config"

// |--------------------------------------------------|
// |  | |                                             |
// |  | |                                             |
// |  | |                                             |
// |YT|Y|                  ChartBox                   |
// |  | |                                             |
// |  | |                                             |
// |  | |                                             |
// |--------------------------------------------------|
// |   	|                X Axis Labels                |
// |    ----------------------------------------------|
// |   	|                X Axis Title                 |
// |--------------------------------------------------|

export type ComputeBoxesOptions = {
	ctx: CanvasRenderingContext2D
	height: number
	includeXAxisLabels: boolean
	includeXAxisTitle: boolean
	includeYAxisLabels: boolean
	includeYAxisTitle: boolean
	width: number
	xAxisGridLineInterval?: GridLineInterval
	yAxisGridLineInterval?: GridLineInterval
	yMax: number
	yMin: number
}

type ComputeBoxesOutput = {
	chartBox: Box
	xAxisLabelsBox: Box
	xAxisTitleBox: Box
	yAxisGridLineInfo: GridLineInfo
	yAxisLabelsBox: Box
	yAxisTitleBox: Box
}

export type Point = {
	x: number
	y: number
}

export type Box = {
	h: number
	w: number
	x: number
	y: number
}

// The interval is k * 10 ** p. k will always be 1, 2, or 5.
export type GridLineInterval = {
	k: number
	p: number
}

export type GridLineInfo = {
	interval: number
	intervalPixels: number
	k: number
	numGridLines: number
	p: number
	start: number
	startPixels: number
}

export function computeBoxes(options: ComputeBoxesOptions): ComputeBoxesOutput {
	let {
		ctx,
		height,
		includeXAxisLabels,
		includeXAxisTitle,
		includeYAxisLabels,
		includeYAxisTitle,
		width,
		yMax,
		yMin,
	} = options
	let {
		bottomPadding,
		fontSize,
		labelPadding,
		leftPadding,
		rightPadding,
		topPadding,
	} = chartConfig

	let chartHeight =
		height -
		(topPadding +
			(includeXAxisLabels ? labelPadding + fontSize : 0) +
			(includeXAxisTitle ? labelPadding + fontSize : 0) +
			bottomPadding)

	let yAxisGridLineInfo = computeYAxisGridLineInfo({
		chartHeight,
		fontSize,
		yAxisGridLineInterval: options.yAxisGridLineInterval,
		yMax,
		yMin,
	})
	let yAxisLabelsWidth = computeAxisLabelsMaxWidth(ctx, yAxisGridLineInfo)

	let chartWidth =
		width -
		(leftPadding +
			(includeYAxisTitle ? fontSize + labelPadding : 0) +
			(includeYAxisLabels ? yAxisLabelsWidth + labelPadding : 0) +
			rightPadding)

	let xAxisLabelsBox = {
		h: includeXAxisLabels ? fontSize : 0,
		w: chartWidth,
		x:
			leftPadding +
			(includeYAxisTitle ? fontSize + labelPadding : 0) +
			(includeYAxisLabels ? yAxisLabelsWidth + labelPadding : 0),
		y: topPadding + chartHeight + (includeXAxisLabels ? labelPadding : 0),
	}

	let xAxisTitleBox = {
		h: includeXAxisTitle ? fontSize : 0,
		w: chartWidth,
		x:
			leftPadding +
			(includeYAxisTitle ? fontSize + labelPadding : 0) +
			(includeYAxisLabels ? yAxisLabelsWidth + labelPadding : 0),
		y:
			topPadding +
			chartHeight +
			(includeXAxisLabels ? labelPadding + fontSize : 0) +
			(includeXAxisTitle ? labelPadding : 0),
	}

	let yAxisTitleBox = {
		h: chartHeight,
		w: fontSize,
		x: leftPadding,
		y: topPadding,
	}

	let yAxisLabelsBox = {
		h: chartHeight,
		w: yAxisLabelsWidth,
		x: leftPadding + (includeYAxisTitle ? fontSize + labelPadding : 0),
		y: topPadding,
	}

	let chartBox = {
		h: chartHeight,
		w: chartWidth,
		x:
			leftPadding +
			(includeYAxisTitle ? fontSize + labelPadding : 0) +
			(includeYAxisLabels ? yAxisLabelsWidth + labelPadding : 0),
		y: topPadding,
	}

	return {
		chartBox,
		xAxisLabelsBox,
		xAxisTitleBox,
		yAxisGridLineInfo,
		yAxisLabelsBox,
		yAxisTitleBox,
	}
}

export function computeGridLineInterval(
	min: number,
	max: number,
	distancePixels: number,
	minGridLineDistancePixels: number,
): GridLineInterval {
	let range = max - min
	let idealN = Math.floor(distancePixels / minGridLineDistancePixels)
	let idealInterval = range / idealN
	let [idealK, idealP] = idealInterval.toExponential().split("e").map(Number)
	let k: number
	let p: number
	if (idealK <= 2) {
		k = 2
		p = idealP
	} else if (idealK <= 5) {
		k = 5
		p = idealP
	} else {
		k = 1
		p = idealP + 1
	}
	return {
		k,
		p,
	}
}

export function computeGridLineInfo(
	min: number,
	max: number,
	distancePixels: number,
	gridLineInterval: GridLineInterval,
): GridLineInfo {
	let range = max - min
	let { k, p } = gridLineInterval
	let interval = k * 10 ** p
	let intervalPixels = distancePixels * (interval / range)
	let start = min - (min % interval) + (min > 0 ? interval : 0)
	let offset = start - min
	let startPixels = (offset / range) * distancePixels
	let numGridLines = Math.floor((range - offset) / interval) + 1
	return {
		interval,
		intervalPixels,
		k,
		numGridLines,
		p,
		start,
		startPixels,
	}
}

type ComputeXAxisGridLineInfoOptions = {
	chartWidth: number
	ctx: CanvasRenderingContext2D
	xAxisGridLineInterval?: GridLineInterval
	xMax: number
	xMin: number
}

export function computeXAxisGridLineInfo(
	options: ComputeXAxisGridLineInfoOptions,
): GridLineInfo {
	let { chartWidth, ctx, xAxisGridLineInterval, xMax, xMin } = options
	let xAxisMinGridLineDistance = 1
	let xAxisGridLineInfo: GridLineInfo | undefined
	if (xAxisGridLineInterval) {
		return computeGridLineInfo(xMin, xMax, chartWidth, xAxisGridLineInterval)
	}
	while (true) {
		xAxisGridLineInterval = computeGridLineInterval(
			xMin,
			xMax,
			chartWidth,
			xAxisMinGridLineDistance,
		)
		xAxisGridLineInfo = computeGridLineInfo(
			xMin,
			xMax,
			chartWidth,
			xAxisGridLineInterval,
		)
		let foundOverlap = false
		for (
			let gridLineIndex = 0;
			gridLineIndex < xAxisGridLineInfo.numGridLines;
			gridLineIndex++
		) {
			let gridLineValue =
				xAxisGridLineInfo.start + gridLineIndex * xAxisGridLineInfo.interval
			let label = formatNumber(gridLineValue)
			let labelWidth = ctx.measureText(label).width
			if (labelWidth > xAxisGridLineInfo.intervalPixels) {
				xAxisMinGridLineDistance = labelWidth
				foundOverlap = true
				break
			}
		}
		if (!foundOverlap) {
			return xAxisGridLineInfo
		}
	}
}

type ComputeYAxisGridLineInfoOptions = {
	chartHeight: number
	fontSize: number
	yAxisGridLineInterval?: GridLineInterval
	yMax: number
	yMin: number
}

export function computeYAxisGridLineInfo(
	options: ComputeYAxisGridLineInfoOptions,
): GridLineInfo {
	let { chartHeight, fontSize, yMax, yMin } = options
	let yAxisGridLineInterval: GridLineInterval = !options.yAxisGridLineInterval
		? computeGridLineInterval(yMin, yMax, chartHeight, fontSize)
		: options.yAxisGridLineInterval
	return computeGridLineInfo(yMin, yMax, chartHeight, yAxisGridLineInterval)
}

function computeAxisLabelsMaxWidth(
	ctx: CanvasRenderingContext2D,
	gridLineInfo: GridLineInfo,
): number {
	return Math.max(
		...times(gridLineInfo.numGridLines, gridLineIndex => {
			let gridLineValue =
				gridLineInfo.start + gridLineIndex * gridLineInfo.interval
			let label = formatNumber(gridLineValue)
			return ctx.measureText(label).width
		}),
	)
}

type DrawXAxisGridLinesOptions = {
	box: Box
	ctx: CanvasRenderingContext2D
	xAxisGridLineInfo: GridLineInfo
}

export function drawXAxisGridLines(options: DrawXAxisGridLinesOptions): void {
	let { box, ctx, xAxisGridLineInfo } = options
	times(xAxisGridLineInfo.numGridLines, gridLineIndex => {
		let gridLineOffsetPixels =
			xAxisGridLineInfo.startPixels +
			gridLineIndex * xAxisGridLineInfo.intervalPixels
		let x = box.x + gridLineOffsetPixels
		ctx.beginPath()
		ctx.strokeStyle = chartColors.current.gridLineColor
		ctx.lineWidth = chartConfig.axisWidth
		ctx.lineCap = "square"
		ctx.moveTo(x, box.y)
		ctx.lineTo(x, box.y + box.h)
		ctx.stroke()
	})
}

type DrawXAxisOptions = {
	box: Box
	ctx: CanvasRenderingContext2D
	yAxisGridLineInfo: GridLineInfo
}

export function drawXAxis(options: DrawXAxisOptions): void {
	let { box, ctx, yAxisGridLineInfo } = options
	times(yAxisGridLineInfo.numGridLines, gridLineIndex => {
		let gridLineValue =
			yAxisGridLineInfo.start + gridLineIndex * yAxisGridLineInfo.interval
		if (gridLineValue === 0) {
			let gridLineOffsetPixels =
				yAxisGridLineInfo.startPixels +
				gridLineIndex * yAxisGridLineInfo.intervalPixels
			let y = box.y + box.h - gridLineOffsetPixels
			ctx.beginPath()
			ctx.strokeStyle = chartColors.current.axisColor
			ctx.lineWidth = chartConfig.axisWidth
			ctx.lineCap = "square"
			ctx.moveTo(box.x, y)
			ctx.lineTo(box.x + box.w, y)
			ctx.stroke()
		}
	})
}

type DrawYAxisGridLinesOptions = {
	box: Box
	ctx: CanvasRenderingContext2D
	yAxisGridLineInfo: GridLineInfo
}

export function drawYAxisGridLines(options: DrawYAxisGridLinesOptions): void {
	let { box, ctx, yAxisGridLineInfo } = options
	times(yAxisGridLineInfo.numGridLines, gridLineIndex => {
		let gridLineOffsetPixels =
			yAxisGridLineInfo.startPixels +
			gridLineIndex * yAxisGridLineInfo.intervalPixels
		let y = box.y + box.h - gridLineOffsetPixels
		ctx.beginPath()
		ctx.strokeStyle = chartColors.current.gridLineColor
		ctx.lineWidth = chartConfig.axisWidth
		ctx.lineCap = "square"
		ctx.moveTo(box.x, y)
		ctx.lineTo(box.x + box.w, y)
		ctx.stroke()
	})
}

type DrawYAxisOptions = {
	box: Box
	ctx: CanvasRenderingContext2D
	xAxisGridLineInfo: GridLineInfo
}

export function drawYAxis(options: DrawYAxisOptions): void {
	let { box, ctx, xAxisGridLineInfo } = options
	times(xAxisGridLineInfo.numGridLines, gridLineIndex => {
		let gridLineValue =
			xAxisGridLineInfo.start + gridLineIndex * xAxisGridLineInfo.interval
		if (gridLineValue !== 0) {
			return
		}
		let gridLineOffsetPixels =
			xAxisGridLineInfo.startPixels +
			gridLineIndex * xAxisGridLineInfo.intervalPixels
		let x = box.x + gridLineOffsetPixels
		ctx.beginPath()
		ctx.strokeStyle = chartColors.current.axisColor
		ctx.lineWidth = chartConfig.axisWidth
		ctx.lineCap = "square"
		ctx.moveTo(x, box.y)
		ctx.lineTo(x, box.y + box.h)
		ctx.stroke()
	})
}

type DrawXAxisLabelsOptions = {
	box: Box
	ctx: CanvasRenderingContext2D
	gridLineInfo: GridLineInfo
	labels?: string[]
	width: number
}

export function drawXAxisLabels(options: DrawXAxisLabelsOptions): void {
	let { box, ctx, gridLineInfo, labels, width } = options
	ctx.fillStyle = chartColors.current.labelColor
	ctx.textBaseline = "bottom"
	ctx.textAlign = "center"

	let previousLabelEndpoint: number | undefined
	for (let i = 0; i < gridLineInfo.numGridLines; i++) {
		let gridLineIndex = i

		let gridLineOffsetPixels =
			gridLineInfo.startPixels + gridLineIndex * gridLineInfo.intervalPixels
		let gridLineValue =
			gridLineInfo.start + gridLineIndex * gridLineInfo.interval
		let label: string
		if (labels) {
			label = labels[gridLineIndex]
		} else {
			label = formatNumber(gridLineValue)
		}

		// Do not draw the label if it will overlap the previous label.
		if (previousLabelEndpoint) {
			if (
				gridLineOffsetPixels - ctx.measureText(label).width / 2 <
				previousLabelEndpoint
			) {
				continue
			}
		}

		// Do not draw the label if it will overflow the chart.
		if (
			box.x + gridLineOffsetPixels - ctx.measureText(label).width / 2 < 0 ||
			box.x + gridLineOffsetPixels + ctx.measureText(label).width / 2 > width
		) {
			break
		}

		ctx.fillText(label, box.x + gridLineOffsetPixels, box.y + box.h)

		// Set the endpoint value of the previous label. This is used to determine if the next label overlaps.
		previousLabelEndpoint =
			gridLineOffsetPixels + ctx.measureText(label).width / 2
	}
}

type DrawYAxisLabelsOptions = {
	box: Box
	ctx: CanvasRenderingContext2D
	fontSize: number
	gridLineInfo: GridLineInfo
	height: number
}

export function drawYAxisLabels(options: DrawYAxisLabelsOptions): void {
	let { box, ctx, fontSize, gridLineInfo, height } = options
	ctx.fillStyle = chartColors.current.labelColor
	ctx.textBaseline = "middle"
	ctx.textAlign = "right"
	times(gridLineInfo.numGridLines, gridLineIndex => {
		let gridLineOffsetPixels =
			gridLineInfo.startPixels + gridLineIndex * gridLineInfo.intervalPixels
		let gridLineValue =
			gridLineInfo.start + gridLineIndex * gridLineInfo.interval
		let label = formatNumber(gridLineValue)
		if (
			box.y + box.h - gridLineOffsetPixels - fontSize / 2 < 0 ||
			box.y + box.h - gridLineOffsetPixels + fontSize / 2 > height
		) {
			return
		}
		ctx.fillText(label, box.x + box.w, box.y + box.h - gridLineOffsetPixels)
	})
}

type DrawXAxisTitleOptions = {
	box: Box
	ctx: CanvasRenderingContext2D
	title?: string
}

export function drawXAxisTitle(options: DrawXAxisTitleOptions): void {
	let { box, ctx, title } = options
	ctx.textAlign = "center"
	ctx.textBaseline = "bottom"
	ctx.fillStyle = chartColors.current.titleColor
	let truncatedTitle = title ? truncateText(ctx, title, box.w) : ""
	ctx.fillText(truncatedTitle, box.x + box.w / 2, box.y + box.h)
}

type DrawYAxisTitleOptions = {
	box: Box
	ctx: CanvasRenderingContext2D
	title?: string
}

export function drawYAxisTitle(options: DrawYAxisTitleOptions): void {
	let { box, ctx, title } = options
	let truncatedTitle = title ? truncateText(ctx, title, box.h) : ""
	ctx.save()
	ctx.translate(box.x + box.w / 2, box.y + box.h / 2)
	ctx.rotate(-Math.PI / 2)
	ctx.textAlign = "center"
	ctx.textBaseline = "middle"
	ctx.fillStyle = chartColors.current.titleColor
	ctx.fillText(truncatedTitle, 0, 0)
	ctx.restore()
}

export type DrawRoundedRectOptions = {
	box: Box
	cornerMask?: number
	ctx: CanvasRenderingContext2D
	fillColor?: string
	radius: number
	strokeColor?: string
	strokeWidth?: number
}

export enum RectCorner {
	TopLeft = 1 << 0,
	TopRight = 1 << 1,
	BottomRight = 1 << 2,
	BottomLeft = 1 << 3,
	None = 0,
	All = TopLeft | TopRight | BottomRight | BottomLeft,
}

export function drawRoundedRect(options: DrawRoundedRectOptions) {
	let {
		box,
		cornerMask,
		ctx,
		fillColor,
		radius,
		strokeColor,
		strokeWidth,
	} = options
	let m = cornerMask ?? RectCorner.All
	let { h, w, x, y } = box
	if (h < 0) {
		h = -h
		y = y - h
	}
	if (w < 0) {
		w = -w
		x = x - w
	}
	ctx.save()
	if (strokeWidth) {
		ctx.lineWidth = strokeWidth
	}
	if (fillColor) {
		ctx.fillStyle = fillColor
	}
	if (strokeColor) {
		ctx.strokeStyle = strokeColor
	}
	ctx.beginPath()
	if (m & RectCorner.TopLeft) {
		ctx.moveTo(x + radius, y)
	} else {
		ctx.moveTo(x, y)
	}
	if (m & RectCorner.TopRight) {
		ctx.lineTo(x + w - radius, y)
		ctx.arcTo(x + w, y, x + w, y + radius, radius)
	} else {
		ctx.lineTo(x + w, y)
	}
	if (m & RectCorner.BottomRight) {
		ctx.lineTo(x + w, y + h - radius)
		ctx.arcTo(x + w, y + h, x + w - radius, y + h, radius)
	} else {
		ctx.lineTo(x + w, y + h)
	}
	if (m & RectCorner.BottomLeft) {
		ctx.lineTo(x + radius, y + h)
		ctx.arcTo(x, y + h, x, y + h - radius, radius)
	} else {
		ctx.lineTo(x, y + h)
	}
	if (m & RectCorner.TopLeft) {
		ctx.lineTo(x, y + radius)
		ctx.arcTo(x, y, x + radius, y, radius)
	} else {
		ctx.lineTo(x, y)
	}
	if (fillColor) {
		ctx.fill()
	}
	if (strokeColor) {
		ctx.stroke()
	}
	ctx.restore()
}

function truncateText(
	ctx: CanvasRenderingContext2D,
	label: string,
	width: number,
) {
	if (ctx.measureText(label).width < width) {
		return label
	}
	let ellipses = "..."
	let truncatedLabel = ""
	let currentLabelTextWidth =
		ctx.measureText(truncatedLabel).width + ctx.measureText(ellipses).width
	let labelChars = label.split("")
	for (let char of labelChars) {
		let charWidth = ctx.measureText(char).width
		if (charWidth + currentLabelTextWidth > width) {
			break
		} else {
			truncatedLabel += char
			currentLabelTextWidth += charWidth
		}
	}
	return truncatedLabel + ellipses
}

function times<T>(n: number, fn: (index: number) => T): T[] {
	let result = []
	for (let i = 0; i < n; i++) {
		result.push(fn(i))
	}
	return result
}

export function formatNumber(value: number | null, maxDigits?: number): string {
	if (value === undefined || value === null) {
		return ""
	}
	let result = value.toPrecision(maxDigits ?? 6)
	// Remove trailing zeros including the decimal point, for example 12345.000.
	result = result.replace(/\.(0*)$/, "")
	// Remove trailing zeros excluding the decimal point, for example .01234500.
	result = result.replace(/\.([0-9]*)([1-9])(0*)$/, ".$1$2")
	return result
}
