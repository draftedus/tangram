import { Box, Point, drawRoundedRect } from './common'
import { chartColors, chartConfig } from './config'

export type TooltipData = {
	color: string
	text: string
}

export type DrawTooltipOptions = {
	centerHorizontal?: boolean
	chartBox: Box
	ctx: CanvasRenderingContext2D
	flipYOffset?: boolean
	origin: Point
	values: TooltipData[]
}

export function drawTooltip(options: DrawTooltipOptions) {
	let { centerHorizontal, chartBox, ctx, flipYOffset, origin, values } = options

	ctx.save()

	let yOffset = 8 // the offset of the tooltip from the origin along the y axis
	let xOffset = centerHorizontal ? 0 : 8 // the offset of the tooltip from the origin along the x axis

	let maxTextWidth = Math.max(
		...options.values.map(point => ctx.measureText(point.text).width),
	)

	let tooltipWindowWidth =
		maxTextWidth +
		2 * chartConfig.tooltipPadding +
		chartConfig.fontSize +
		chartConfig.tooltipPadding

	let tooltipWindowHeight =
		2 * chartConfig.tooltipPadding +
		chartConfig.fontSize * values.length +
		chartConfig.tooltipPadding * (values.length - 1)

	let tooltipWindowX = Math.min(
		Math.max(
			chartBox.x,
			centerHorizontal
				? options.origin.x - tooltipWindowWidth / 2
				: options.origin.x + xOffset,
		),
		chartBox.x + chartBox.w - tooltipWindowWidth - xOffset,
	)

	let tooltipWindowY: number
	// if the bar chart bar is negative, display the tooltip below the bar
	if (flipYOffset) {
		tooltipWindowY =
			// if displaying the tooltip below is out of bounds of the chart, display it above
			origin.y + yOffset + tooltipWindowHeight > chartBox.y + chartBox.h
				? origin.y - yOffset - tooltipWindowHeight
				: origin.y + yOffset
	} else {
		tooltipWindowY =
			// if displaying the tooltip above is out of bounds of the chart, display it below
			origin.y - yOffset - tooltipWindowHeight < 0
				? origin.y + yOffset
				: origin.y - yOffset - tooltipWindowHeight
	}

	// draw tooltip window
	ctx.shadowColor = chartColors.current.tooltipShadowColor
	ctx.shadowBlur = chartConfig.tooltipShadowBlur
	ctx.shadowOffsetX = 0
	ctx.shadowOffsetY = 0
	drawRoundedRect({
		box: {
			h: tooltipWindowHeight,
			w: tooltipWindowWidth,
			x: tooltipWindowX,
			y: tooltipWindowY,
		},
		ctx,
		fillColor: chartColors.current.tooltipBackgroundColor,
		radius: 4,
	})

	values.forEach((value, i) => {
		// draw color indicator
		let box = {
			h: chartConfig.fontSize,
			w: chartConfig.fontSize,
			x: tooltipWindowX + chartConfig.tooltipPadding,
			y:
				tooltipWindowY +
				chartConfig.tooltipPadding +
				(chartConfig.fontSize + chartConfig.tooltipPadding) * i,
		}
		drawRoundedRect({
			box,
			ctx,
			fillColor: value.color,
			radius: chartConfig.tooltipBorderRadius,
		})
		// draw label
		ctx.fillStyle = chartColors.current.textColor
		let text = value.text
		let tooltipTextX =
			tooltipWindowX +
			chartConfig.tooltipPadding +
			chartConfig.fontSize +
			chartConfig.tooltipPadding
		let tooltipTextY =
			tooltipWindowY +
			chartConfig.tooltipPadding +
			(chartConfig.fontSize + chartConfig.tooltipPadding) * i
		ctx.textBaseline = 'top'
		ctx.fillText(text, tooltipTextX, tooltipTextY)
	})

	ctx.restore()
}
