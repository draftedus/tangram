import {
	chartColors,
	chartConfig,
	darkChartColors,
	lightChartColors,
} from './config'

export type Chart<Options> = {
	destroy: () => void
	draw: (options: Options) => void
}

export type DrawFunction<Options, Info, HoverRegionInfo> = (
	ctx: CanvasRenderingContext2D,
	options: Options,
) => DrawOutput<Info, HoverRegionInfo>

export type DrawOutput<OverlayInfo, HoverRegionInfo> = {
	hoverRegions: Array<HoverRegion<HoverRegionInfo>>
	overlayInfo: OverlayInfo
}

type DrawOverlayFunction<Info, HoverRegionInfo> = (
	options: DrawOverlayOptions<Info, HoverRegionInfo>,
) => void

export type DrawOverlayOptions<Info, HoverRegionInfo> = {
	activeHoverRegions: Array<ActiveHoverRegion<HoverRegionInfo>>
	ctx: CanvasRenderingContext2D
	info: Info
}

export type HoverRegion<HoverRegionInfo> = {
	distance: (x: number, y: number) => number
	hitTest: (x: number, y: number) => boolean
	info: HoverRegionInfo
}

export type ActiveHoverRegion<HoverRegionInfo> = {
	distance: number
	info: HoverRegionInfo
}

type ChartState<Options, OverlayInfo, HoverRegionInfo> = {
	activeHoverRegions: Array<ActiveHoverRegion<HoverRegionInfo>> | null
	hoverRegions: Array<HoverRegion<HoverRegionInfo>> | null
	options: Options | null
	overlayInfo: OverlayInfo | null
}

export function createChart<Options, OverlayInfo, HoverRegionInfo>(
	container: HTMLElement,
	drawChart: DrawFunction<Options, OverlayInfo, HoverRegionInfo>,
	drawChartOverlay?: DrawOverlayFunction<OverlayInfo, HoverRegionInfo>,
): Chart<Options> {
	let state: ChartState<Options, OverlayInfo, HoverRegionInfo> = {
		activeHoverRegions: null,
		hoverRegions: null,
		options: null,
		overlayInfo: null,
	}

	container.style.position = 'relative'

	let chartCanvas = document.createElement('canvas')
	chartCanvas.style.position = 'absolute'
	chartCanvas.style.top = '0'
	chartCanvas.style.bottom = '0'
	chartCanvas.style.left = '0'
	chartCanvas.style.right = '0'
	container.appendChild(chartCanvas)

	let overlayCanvas = document.createElement('canvas')
	overlayCanvas.style.position = 'absolute'
	overlayCanvas.style.top = '0'
	overlayCanvas.style.bottom = '0'
	overlayCanvas.style.left = '0'
	overlayCanvas.style.right = '0'
	container.appendChild(overlayCanvas)

	function updateActiveHoverRegions(x: number, y: number) {
		if (!state.hoverRegions) throw Error()
		state.activeHoverRegions = []
		for (let hoverRegion of state.hoverRegions) {
			if (hoverRegion.hitTest(x, y)) {
				state.activeHoverRegions.push({
					distance: hoverRegion.distance(x, y),
					info: hoverRegion.info,
				})
			}
		}
		renderChartOverlay()
	}

	function onMouseEvent(event: MouseEvent) {
		let canvasClientRect = chartCanvas.getBoundingClientRect()
		let x = event.clientX - canvasClientRect.left
		let y = event.clientY - canvasClientRect.top
		updateActiveHoverRegions(x, y)
	}
	overlayCanvas.addEventListener('mouseenter', onMouseEvent)
	overlayCanvas.addEventListener('mouseleave', onMouseEvent)
	overlayCanvas.addEventListener('mousemove', onMouseEvent)

	function onTouchEvent(event: TouchEvent) {
		let canvasClientRect = chartCanvas.getBoundingClientRect()
		let x = event.touches[0].clientX - canvasClientRect.left
		let y = event.touches[0].clientY - canvasClientRect.top
		updateActiveHoverRegions(x, y)
	}
	overlayCanvas.addEventListener('touchstart', onTouchEvent)

	window.addEventListener('resize', render)
	let colorSchemeMediaQuery = matchMedia(`(prefers-color-scheme: dark)`)
	colorSchemeMediaQuery.addListener(render)
	let dprMediaQuery = matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`)
	dprMediaQuery.addListener(render)

	function renderChart() {
		if (!state.options) throw Error()
		let width = container.clientWidth
		let height = container.clientHeight
		let dpr = window.devicePixelRatio
		chartCanvas.width = width * dpr
		chartCanvas.height = height * dpr
		chartCanvas.style.width = width.toString() + 'px'
		chartCanvas.style.height = height.toString() + 'px'
		chartColors.current = colorSchemeMediaQuery.matches
			? darkChartColors
			: lightChartColors
		let ctx = chartCanvas.getContext('2d')
		if (!ctx) {
			throw Error()
		}
		ctx.scale(dpr, dpr)
		ctx.clearRect(0, 0, width, height)
		ctx.font = chartConfig.font
		let output = drawChart(ctx, state.options)
		state.hoverRegions = output.hoverRegions
		state.overlayInfo = output.overlayInfo
	}

	function renderChartOverlay() {
		if (!state.overlayInfo) throw Error()
		let width = container.clientWidth
		let height = container.clientHeight
		let dpr = window.devicePixelRatio
		overlayCanvas.width = width * dpr
		overlayCanvas.height = height * dpr
		overlayCanvas.style.width = width.toString() + 'px'
		overlayCanvas.style.height = height.toString() + 'px'
		chartColors.current = colorSchemeMediaQuery.matches
			? darkChartColors
			: lightChartColors
		let ctx = overlayCanvas.getContext('2d')
		if (!ctx) {
			throw Error()
		}
		ctx.scale(dpr, dpr)
		ctx.clearRect(0, 0, width, height)
		ctx.font = chartConfig.font
		drawChartOverlay?.({
			activeHoverRegions: state.activeHoverRegions ?? [],
			ctx,
			info: state.overlayInfo,
		})
	}

	function render() {
		renderChart()
		renderChartOverlay()
	}

	function destroy() {
		window.removeEventListener('resize', render)
		colorSchemeMediaQuery.removeListener(render)
		dprMediaQuery.removeListener(render)
		container.removeChild(chartCanvas)
		container.removeChild(overlayCanvas)
	}

	function draw(newOptions: Options) {
		state.options = newOptions
		render()
	}

	return {
		destroy,
		draw,
	}
}
