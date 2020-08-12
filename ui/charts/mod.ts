export type {
	BarChartData,
	BarChartSeries,
	BarChartOptions as DrawBarChartOptions,
} from './bar_chart'
export type {
	BoxChartData,
	BoxChartPoint,
	BoxChartSeries,
	BoxChartOptions as DrawBoxChartOptions,
} from './box_chart'
export type {
	ShapChartOptions as DrawShapChartOptions,
	ShapChartData,
	ShapChartSeries,
	ShapValue,
} from './shap_chart'
export type {
	LineChartOptions as DrawLineChartOptions,
	LineChartData,
	LineChartSeries,
} from './line_chart'
export { createBarChart } from './bar_chart'
export { createBoxChart } from './box_chart'
export { createLineChart, LineStyle, PointStyle } from './line_chart'
export { createShapChart } from './shap_chart'
export { chartConfig, lightChartColors, darkChartColors } from './config'
export * from './components'

import { createBarChart } from './bar_chart'
import { createBoxChart } from './box_chart'
import { Chart } from './chart'
import { createLineChart } from './line_chart'
import { createShapChart } from './shap_chart'

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

export function hydrateShapChart(id: string) {
	hydrateChart(id, createShapChart)
}
