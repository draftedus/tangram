import './number_comparison_chart.css'
import { Token } from './token'
import { cx } from './util'
import { h } from 'preact'

type NumberComparisonChartProps = {
	colorA: string
	colorB: string
	title: string
	valueA: number | null
	valueATitle: string
	valueB: number | null
	valueBTitle: string
	valueFormatter?: (value: number) => string
}

function defaultValueFormatter(value: number) {
	return value === null ? 'N/A' : value.toString()
}

export function NumberComparisonChart(props: NumberComparisonChartProps) {
	let valueFormatter = props.valueFormatter ?? defaultValueFormatter
	let differenceString =
		props.valueA === null || props.valueB === null
			? 'N/A'
			: props.valueB - props.valueA > 0
			? '+' + valueFormatter(props.valueB - props.valueA)
			: props.valueB - props.valueA < 0
			? valueFormatter(props.valueB - props.valueA)
			: 'equal'
	let differenceClass = cx(
		'number-comparison-difference',
		props.valueB !== null &&
			props.valueA !== null &&
			props.valueB - props.valueA > 0
			? 'number-comparison-positive'
			: props.valueB !== null &&
			  props.valueA !== null &&
			  props.valueB - props.valueA < 0
			? 'number-comparison-negative'
			: 'number-comparison-equals',
	)
	return (
		<div class="number-comparison-wrapper">
			<div class="number-comparison-title">{props.title}</div>
			<div class={differenceClass}>{differenceString}</div>
			<div class="number-comparison-inner-wrapper">
				<div class="number-comparison-value">
					{props.valueA !== null ? valueFormatter(props.valueA) : 'N/A'}
				</div>
				<div class="number-comparison-value">
					{props.valueB !== null ? valueFormatter(props.valueB) : 'N/A'}
				</div>
				<div>
					<Token color={props.colorA}>{props.valueATitle}</Token>
				</div>
				<div>
					<Token color={props.colorB}>{props.valueBTitle}</Token>
				</div>
			</div>
		</div>
	)
}
