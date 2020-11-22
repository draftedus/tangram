import { createBarChart } from "./bar_chart"
import { createBoxChart } from "./box_chart"
import { Chart } from "./chart"
import { createFeatureContributionsChart } from "./feature_contributions_chart"
import { createLineChart } from "./line_chart"

export type { BarChartSeries, BarChartOptions } from "./bar_chart"
export type {
	BoxChartPoint,
	BoxChartSeries,
	BoxChartOptions,
} from "./box_chart"
export type {
	FeatureContributionsChartOptions,
	FeatureContributionsChartSeries,
	FeatureContributionsChartValue,
} from "./feature_contributions_chart"
export type { LineChartOptions, LineChartSeries } from "./line_chart"
export { createBarChart } from "./bar_chart"
export { createBoxChart } from "./box_chart"
export { createLineChart, LineStyle, PointStyle } from "./line_chart"
export { createFeatureContributionsChart } from "./feature_contributions_chart"
export { chartConfig, lightChartColors, darkChartColors } from "./config"
export * from "./components"

export function hydrateChart<Options>(
	id: string,
	create: (element: HTMLElement, options?: Options) => Chart<Options>,
) {
	let container = document.getElementById(id)
	if (!container) throw Error()
	let optionsJson = container.dataset.options
	if (!optionsJson) throw Error()
	let options = JSON.parse(optionsJson)
	let chart = create(container)
	chart.draw(options)
}

export function hydrateBarChart(id: string) {
	hydrateChart(id, createBarChart)
}

export function hydrateBoxChart(id: string) {
	hydrateChart(id, createBoxChart)
}

export function hydrateLineChart(id: string) {
	hydrateChart(id, createLineChart)
}

export function hydrateFeatureContributionsChart(id: string) {
	hydrateChart(id, createFeatureContributionsChart)
}
