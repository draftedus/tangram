import { Children, h, useEffect, useRef } from '../deps'
import { DrawBarChartOptions, createBarChart } from './bar_chart'
import { DrawBoxChartOptions, createBoxChart } from './box_chart'
import { chartConfig } from './config'
import { DrawLineChartOptions, createLineChart } from './line_chart'
import { DrawShapChartOptions, createShapChart } from './shap_chart'

export type BarChartProps = DrawBarChartOptions & {
	id?: string
	title?: string
}

export function BarChart(props: BarChartProps) {
	let containerRef = useRef<HTMLDivElement | null>(null)
	let chartRef = useRef<ReturnType<typeof createBarChart> | null>(null)

	useEffect(() => {
		if (!containerRef.current) throw Error()
		chartRef.current = createBarChart(containerRef.current)
		return () => chartRef.current?.destroy()
	}, [])

	useEffect(() => chartRef.current?.draw(props))

	let containerStyle = {
		paddingTop: '50%',
		width: '100%',
	}

	return (
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			{!props.hideLegend && <ChartLegend items={props.data} />}
			<div
				data-chart-type="bar"
				data-options={props.id && JSON.stringify(props)}
				id={props.id}
				ref={containerRef}
				style={containerStyle}
			>
				<noscript>
					<div class="chart-noscript">
						{'Please enable JavaScript to view charts.'}
					</div>
				</noscript>
			</div>
		</div>
	)
}

export type BoxChartProps = DrawBoxChartOptions & {
	id?: string
	title?: string
}

export function BoxChart(props: BoxChartProps) {
	let containerRef = useRef<HTMLDivElement | null>(null)
	let chartRef = useRef<ReturnType<typeof createBoxChart> | null>(null)

	useEffect(() => {
		if (!containerRef.current) throw Error()
		chartRef.current = createBoxChart(containerRef.current)
		return () => chartRef.current?.destroy()
	}, [])

	useEffect(() => chartRef.current?.draw(props))

	let containerStyle = {
		paddingTop: '50%',
		width: '100%',
	}

	return (
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			{!props.hideLegend && <ChartLegend items={props.data} />}
			<div
				data-chart-type="box"
				data-options={props.id && JSON.stringify(props)}
				id={props.id}
				ref={containerRef}
				style={containerStyle}
			>
				<noscript>
					<div class="chart-noscript">
						{'Please enable JavaScript to view charts.'}
					</div>
				</noscript>
			</div>
		</div>
	)
}

export type LineChartProps = DrawLineChartOptions & {
	id?: string
	title?: string
}

export function LineChart(props: LineChartProps) {
	let containerRef = useRef<HTMLDivElement | null>(null)
	let chartRef = useRef<ReturnType<typeof createLineChart> | null>(null)

	useEffect(() => {
		if (!containerRef.current) throw Error()
		chartRef.current = createLineChart(containerRef.current)
		return () => chartRef.current?.destroy()
	}, [])

	useEffect(() => chartRef.current?.draw(props))

	let containerStyle = {
		paddingTop: '50%',
		width: '100%',
	}

	return (
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			{!props.hideLegend && <ChartLegend items={props.data} />}
			<div
				data-chart-type="line"
				data-options={props.id && JSON.stringify(props)}
				id={props.id}
				ref={containerRef}
				style={containerStyle}
			>
				<noscript>
					<div class="chart-noscript">
						{'Please enable JavaScript to view charts.'}
					</div>
				</noscript>
			</div>
		</div>
	)
}

export type ShapChartProps = DrawShapChartOptions & {
	id?: string
	title?: string
}

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

	let containerStyle = {
		height: `${height}px`,
		width: '100%',
	}

	return (
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			<div
				data-chart-type="shap"
				data-options={props.id && JSON.stringify(props)}
				id={props.id}
				ref={containerRef}
				style={containerStyle}
			>
				<noscript>
					<div class="chart-noscript">
						{'Please enable JavaScript to view charts.'}
					</div>
				</noscript>
			</div>
		</div>
	)
}

type ChartTitleProps = { children?: Children }

export function ChartTitle(props: ChartTitleProps) {
	return <div class="chart-title">{props.children}</div>
}

type ChartLegendProps = {
	items: LegendItem[]
}

export type LegendItem = {
	color: string
	title: string
}

export function ChartLegend(props: ChartLegendProps) {
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

function ChartLegendItemCell(props: LegendItemCellProps) {
	let style = {
		backgroundColor: props.color,
	}
	return (
		<div class="chart-legend-item">
			<div class="chart-legend-indicator" style={style} />
			<div class="chart-legend-title">{props.title}</div>
		</div>
	)
}
