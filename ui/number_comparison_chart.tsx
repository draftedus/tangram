import { css, cx, h, useCss } from './deps'
import { colors, mobile, variables } from './theme'
import { Token } from './token'

type NumberComparisonChartProps = {
	colorA: string
	colorB: string
	textColorA?: string
	textColorB?: string
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

let numberComparisonWrapper = css({
	[`.number-comparison-wrapper`]: {
		color: variables.colors.text,
		display: 'grid',
		gridGap: '1rem',
		justifyItems: 'center',
		textAlign: 'center',
	},
	[mobile]: {
		[`.number-comparison-wrapper`]: {
			gridGap: '.5rem',
		},
	},
})

let numberComparisonInnerWrapper = css({
	[`.number-comparison-inner-wrapper`]: {
		alignItems: 'center',
		display: 'grid',
		grid: '1fr 1fr / 1fr 1fr',
		gridColumnGap: '1rem',
		gridRowGap: '.5rem',
		justifyItems: 'center',
	},
	[mobile]: {
		[`.number-comparison-inner-wrapper`]: {
			gridColumnGap: '0.5rem',
		},
	},
})

let titleCss = css({
	[`.number-comparison-title`]: {
		fontSize: '1.25rem',
	},
	[mobile]: {
		[`.number-comparison-title`]: {
			fontSize: '1rem',
		},
	},
})

let valueCss = css({
	[`.number-comparison-value`]: {
		fontSize: '2rem',
	},
	[mobile]: {
		[`.number-comparison-value`]: {
			fontSize: '1.25rem',
		},
	},
})

let positiveCss = css({
	[`.number-comparison-positive`]: { color: colors.green, display: 'flex' },
})

let negativeCss = css({
	[`.number-comparison-negative`]: { color: colors.red },
})

let equalsCss = css({
	[`.number-comparison-equals`]: { color: colors.gray },
})

let differenceCss = css({
	[`.number-comparison-difference`]: {
		alignContent: 'center',
		display: 'grid',
		gridAutoFlow: 'column',
	},
})

export function NumberComparisonChart(props: NumberComparisonChartProps) {
	useCss(
		numberComparisonWrapper,
		numberComparisonInnerWrapper,
		titleCss,
		valueCss,
		differenceCss,
		positiveCss,
		negativeCss,
		equalsCss,
	)
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
					<Token color={props.colorA} textColor={props.textColorA}>
						{props.valueATitle}
					</Token>
				</div>
				<div>
					<Token color={props.colorB} textColor={props.textColorB}>
						{props.valueBTitle}
					</Token>
				</div>
			</div>
		</div>
	)
}
