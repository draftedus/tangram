import { Children, css, h, useCss, useEffect, useRef } from '../deps'
import { DrawBarChartOptions, createBarChart } from './bar_chart'
import { DrawBoxChartOptions, createBoxChart } from './box_chart'
import { chartConfig } from './config'
import { DrawLineChartOptions, createLineChart } from './line_chart'
import { DrawShapChartOptions, createShapChart } from './shap_chart'

export type BarChartProps = DrawBarChartOptions & { title?: string }

let wrapperCss = css({
	[`.chart-wrapper`]: { display: 'grid', gridRowGap: '1rem' },
})

export function BarChart(props: BarChartProps) {
	let containerRef = useRef<HTMLDivElement | null>(null)
	let chartRef = useRef<ReturnType<typeof createBarChart> | null>(null)

	useEffect(() => {
		if (!containerRef.current) throw Error()
		chartRef.current = createBarChart(containerRef.current)
		return () => chartRef.current?.destroy()
	}, [])

	useEffect(() => chartRef.current?.draw(props))

	useCss(wrapperCss)
	let containerStyle = {
		paddingTop: '50%',
		width: '100%',
	}

	return (
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			<ChartLegend items={props.data} />
			<div ref={containerRef} style={containerStyle} />
		</div>
	)
}

export type BoxChartProps = DrawBoxChartOptions & { title?: string }

export function BoxChart(props: BoxChartProps) {
	let containerRef = useRef<HTMLDivElement | null>(null)
	let chartRef = useRef<ReturnType<typeof createBoxChart> | null>(null)

	useEffect(() => {
		if (!containerRef.current) throw Error()
		chartRef.current = createBoxChart(containerRef.current)
		return () => chartRef.current?.destroy()
	}, [])

	useEffect(() => chartRef.current?.draw(props))

	useCss(wrapperCss)

	let containerStyle = {
		paddingTop: '50%',
		width: '100%',
	}

	return (
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			<ChartLegend items={props.data} />
			<div ref={containerRef} style={containerStyle} />
		</div>
	)
}

export type LineChartProps = DrawLineChartOptions & { title?: string }

export function LineChart(props: LineChartProps) {
	let containerRef = useRef<HTMLDivElement | null>(null)
	let chartRef = useRef<ReturnType<typeof createLineChart> | null>(null)

	useEffect(() => {
		if (!containerRef.current) throw Error()
		chartRef.current = createLineChart(containerRef.current)
		return () => chartRef.current?.destroy()
	}, [])

	useEffect(() => chartRef.current?.draw(props))

	useCss(wrapperCss)

	let containerStyle = {
		paddingTop: '50%',
		width: '100%',
	}

	return (
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			<ChartLegend items={props.data} />
			<div ref={containerRef} style={containerStyle} />
		</div>
	)
}

export type ShapChartProps = DrawShapChartOptions & { title?: string }

export function ShapChart(props: ShapChartProps) {
	let containerRef = useRef<HTMLDivElement | null>(null)
	let chartRef = useRef<ReturnType<typeof createShapChart> | null>(null)

	useEffect(() => {
		if (!containerRef.current) throw Error()
		chartRef.current = createShapChart(containerRef.current)
		return () => chartRef.current?.destroy()
	}, [])

	useEffect(() => chartRef.current?.draw(props))

	let innerChartHeight =
		props.data.length * chartConfig.shapGroupHeight +
		(props.data.length - 1) * chartConfig.shapGroupGap
	let { bottomPadding, fontSize, labelPadding, topPadding } = chartConfig
	let height =
		innerChartHeight +
		topPadding +
		labelPadding +
		fontSize +
		(props.includeXAxisTitle ? labelPadding + fontSize : 0) +
		labelPadding +
		fontSize +
		bottomPadding

	useCss(wrapperCss)

	let containerStyle = {
		height: `${height}px`,
		width: '100%',
	}

	return (
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			<div ref={containerRef} style={containerStyle} />
		</div>
	)
}

type ChartTitleProps = { children?: Children }
let chartTitleCss = css({
	[`.chart-title`]: { fontSize: '1.25rem', textAlign: 'center' },
})

export function ChartTitle(props: ChartTitleProps) {
	useCss(chartTitleCss)
	return <div class="chart-title">{props.children}</div>
}

type ChartLegendProps = {
	items: LegendItem[]
}

export type LegendItem = {
	color: string
	title: string
}

let chartLegendWrapperCss = css({
	[`.chart-legend-wrapper`]: {
		alignItems: 'center',
		display: 'flex',
		flexWrap: 'wrap',
		justifyContent: 'center',
	},
})

export function ChartLegend(props: ChartLegendProps) {
	useCss(chartLegendWrapperCss)
	return (
		<div class="chart-legend-wrapper">
			{props.items.map(category => (
				<ChartLegendItemCell
					color={category.color}
					key={category.title}
					title={category.title}
				/>
			))}
		</div>
	)
}

type LegendItemCellProps = {
	color: string
	title: string
}

let chartLegendItemCss = css({
	[`.chart-legend-item`]: {
		alignItems: 'center',
		display: 'grid',
		grid: 'auto / auto auto',
		gridColumnGap: '0.5rem',
		justifyContent: 'start',
		margin: '0.5rem',
	},
})

let chartLegendIndicatorCss = css({
	[`.chart-legend-indicator`]: {
		borderRadius: '4px',
		boxSizing: 'border-box',
		height: '1rem',
		width: '1rem',
	},
})

let chartLegendTitleCss = css({
	[`.chart-legend-title`]: { whiteSpace: 'nowrap' },
})

function ChartLegendItemCell(props: LegendItemCellProps) {
	useCss(chartLegendItemCss, chartLegendIndicatorCss, chartLegendTitleCss)
	let style = {
		backgroundColor: props.color,
	}
	return (
		<div class="chart-legend-item">
			<div class="chart-legend-indicator" style={style} />
			<div class="chart-legend-titel">{props.title}</div>
		</div>
	)
}
