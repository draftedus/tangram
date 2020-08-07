import { cx, h } from './deps'
import { Token } from './token'

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
	let difference = Number(props.valueB) - Number(props.valueA)
	return (
		<div class="number-comparison-wrapper">
			<div class="number-comparison-title">{props.title}</div>
			<div
				class={cx(
					'number-comparison-difference',
					difference > 0
						? 'number-comparison-positive'
						: difference < 0
						? 'number-comparison-negative'
						: 'number-comparison-equals',
				)}
			>
				{props.valueA === null || props.valueB === null
					? 'N/A'
					: difference == 0
					? 'equal'
					: valueFormatter(difference)}
			</div>
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
