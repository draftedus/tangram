import { h } from './deps'

type NumberChartProps = {
	title: string
	value: string
}

export function NumberChart(props: NumberChartProps) {
	return (
		<div class="number-chart-wrapper">
			<div class="number-chart-value">{props.value}</div>
			<div class="number-chart-title">{props.title}</div>
		</div>
	)
}
