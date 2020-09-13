import { Point } from './common'
import { chartColors, chartConfig } from './config'

export type TooltipData = {
	color: string
	text: string
}

type DrawTooltipOptions = {
	centerHorizontal?: boolean
	container: HTMLElement
	flipYOffset?: boolean
	origin: Point
	values: TooltipData[]
}

export function drawTooltip(options: DrawTooltipOptions) {
	let {
		centerHorizontal,
		container,
		origin: { x, y },
		values,
	} = options
	let tooltipWrapper = document.createElement('div')
	tooltipWrapper.style.position = 'relative'
	tooltipWrapper.style.display = 'grid'
	tooltipWrapper.style.gridGap = '0.5rem'
	tooltipWrapper.style.grid = 'auto / auto auto'
	tooltipWrapper.style.alignItems = 'center'
	tooltipWrapper.style.boxShadow = `0 0 ${chartConfig.tooltipShadowBlur} ${chartColors.current.tooltipShadowColor}`
	tooltipWrapper.style.width = 'max-content'
	tooltipWrapper.style.top = `calc(${y}px - 8px)`
	tooltipWrapper.style.borderRadius = `${chartConfig.tooltipBorderRadius}px`
	tooltipWrapper.style.font = chartConfig.font
	tooltipWrapper.style.backgroundColor =
		chartColors.current.tooltipBackgroundColor
	tooltipWrapper.style.padding = `${chartConfig.tooltipPadding}px`
	tooltipWrapper.style.zIndex = '2'
	if (centerHorizontal) {
		tooltipWrapper.style.left = `${x}px`
		tooltipWrapper.style.transform = 'translateX(-50%) translateY(-100%)'
	} else {
		tooltipWrapper.style.left = `calc(${x}px + 8px)`
		tooltipWrapper.style.transform = 'translateY(-100%)'
	}
	values.forEach(value => {
		let tooltipRect = document.createElement('div')
		tooltipRect.style.backgroundColor = value.color
		tooltipRect.style.borderRadius = `${chartConfig.tooltipBorderRadius}px`
		tooltipRect.style.width = `${chartConfig.fontSize}px`
		tooltipRect.style.height = `${chartConfig.fontSize}px`
		let tooltip = document.createElement('div')
		tooltip.innerText = value.text
		tooltipWrapper.appendChild(tooltipRect)
		tooltipWrapper.appendChild(tooltip)
	})
	container.appendChild(tooltipWrapper)
	// if the tooltip is not visible, place it elsewhere
	let boundingRect = tooltipWrapper.getBoundingClientRect()
	let windowWidth = window.innerWidth
	let overflowRight = boundingRect.x + boundingRect.width - windowWidth
	let overflowLeft = -boundingRect.x
	let padding = '16px'
	if (overflowRight > 0) {
		// translate by the amount that it is overflowing
		tooltipWrapper.style.transform = `translateX(calc(-50% - ${overflowRight}px - ${padding})) translateY(-100%)`
	} else if (overflowLeft > 0) {
		tooltipWrapper.style.transform = `translateX(calc(-50% + ${overflowLeft}px + ${padding})) translateY(-100%)`
	}
}
