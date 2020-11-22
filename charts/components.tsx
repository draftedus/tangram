import { BarChartOptions, createBarChart } from "./bar_chart"
import { BoxChartOptions, createBoxChart } from "./box_chart"
import "./charts.css"
import { chartConfig } from "./config"
import {
	FeatureContributionsChartOptions,
	createFeatureContributionsChart,
} from "./feature_contributions_chart"
import { LineChartOptions, createLineChart } from "./line_chart"
import { ComponentChildren, h } from "preact"
import { useEffect, useRef } from "preact/hooks"

export type BarChartProps = BarChartOptions & {
	class?: string
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
		paddingTop: "50%",
		width: "100%",
	}

	let legendItems: LegendItem[] = []
	for (let series of props.series) {
		if (series.title !== undefined) {
			legendItems.push({
				color: series.color,
				title: series.title,
			})
		}
	}

	let hideLegend = props.hideLegend ?? legendItems.length <= 1

	return (
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			{!hideLegend && <ChartLegend items={legendItems} />}
			<div
				class={props.class}
				data-chart-type="bar"
				data-options={props.id && JSON.stringify(props)}
				id={props.id}
				ref={containerRef}
				style={containerStyle}
			>
				<noscript>
					<div class="chart-noscript">
						{"Please enable JavaScript to view charts."}
					</div>
				</noscript>
			</div>
		</div>
	)
}

export type BoxChartProps = BoxChartOptions & {
	class?: string
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
		paddingTop: "50%",
		width: "100%",
	}

	// If hideLegend is undefined, only show the legend when there is more than one series.
	let hideLegend = props.hideLegend
	if (hideLegend === undefined) {
		hideLegend = props.series.length <= 1
	}

	return (
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			{!hideLegend && <ChartLegend items={props.series} />}
			<div
				class={props.class}
				data-chart-type="box"
				data-options={props.id && JSON.stringify(props)}
				id={props.id}
				ref={containerRef}
				style={containerStyle}
			>
				<noscript>
					<div class="chart-noscript">
						{"Please enable JavaScript to view charts."}
					</div>
				</noscript>
			</div>
		</div>
	)
}

export type LineChartProps = LineChartOptions & {
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
		paddingTop: "50%",
		width: "100%",
	}

	// If hideLegend is undefined, only show the legend when there is more than one series.
	let hideLegend = props.hideLegend
	if (hideLegend === undefined) {
		hideLegend = props.series.length <= 1
	}

	return (
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			{!hideLegend && <ChartLegend items={props.series} />}
			<div
				data-chart-type="line"
				data-options={props.id && JSON.stringify(props)}
				id={props.id}
				ref={containerRef}
				style={containerStyle}
			>
				<noscript>
					<div class="chart-noscript">
						{"Please enable JavaScript to view charts."}
					</div>
				</noscript>
			</div>
		</div>
	)
}

export type FeatureContributionsChartProps = FeatureContributionsChartOptions & {
	id?: string
	title?: string
}

export function FeatureContributionsChart(
	props: FeatureContributionsChartProps,
) {
	let containerRef = useRef<HTMLDivElement | null>(null)
	let chartRef = useRef<ReturnType<
		typeof createFeatureContributionsChart
	> | null>(null)

	useEffect(() => {
		if (!containerRef.current) throw Error()
		chartRef.current = createFeatureContributionsChart(containerRef.current)
		return () => chartRef.current?.destroy()
	}, [])

	useEffect(() => chartRef.current?.draw(props))

	let innerChartHeight =
		props.series.length * chartConfig.featureContributionsSeriesHeight +
		(props.series.length - 1) * chartConfig.featureContributionsSeriesGap
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
		width: "100%",
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
						{"Please enable JavaScript to view charts."}
					</div>
				</noscript>
			</div>
		</div>
	)
}

type ChartTitleProps = { children?: ComponentChildren }

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
			{props.items.map(item => (
				<ChartLegendItemCell
					color={item.color}
					key={item.title}
					title={item.title}
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
