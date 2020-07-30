import { css, cssClass, cx, h, useCss } from './deps'
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

let containerClass = cssClass()
let containerCss = css({
	[`.${containerClass}`]: {
		color: variables.colors.text,
		display: 'grid',
		gridGap: '1rem',
		justifyItems: 'center',
		textAlign: 'center',
	},
	[mobile]: {
		[`.${containerClass}`]: {
			gridGap: '.5rem',
		},
	},
})

let numberComparisonContainerClass = cssClass()
let numberComparisonContainerCss = css({
	[`.${numberComparisonContainerClass}`]: {
		alignItems: 'center',
		display: 'grid',
		grid: '1fr 1fr / 1fr 1fr',
		gridColumnGap: '1rem',
		gridRowGap: '.5rem',
		justifyItems: 'center',
	},
	[mobile]: {
		[`.${numberComparisonContainerClass}`]: {
			gridColumnGap: '0.5rem',
		},
	},
})

let titleClass = cssClass()
let titleCss = css({
	[`.${titleClass}`]: {
		fontSize: '1.25rem',
	},
	[mobile]: {
		[`.${titleClass}`]: {
			fontSize: '1rem',
		},
	},
})

let valueClass = cssClass()
let valueCss = css({
	[`.${valueClass}`]: {
		fontSize: '2rem',
	},
	[mobile]: {
		[`.${valueClass}`]: {
			fontSize: '1.25rem',
		},
	},
})

let positiveClass = cssClass()
let positiveCss = css({
	[`.${positiveClass}`]: { color: colors.green, display: 'flex' },
})

let negativeClass = cssClass()
let negativeCss = css({
	[`.${negativeClass}`]: { color: colors.red },
})

let equalsClass = cssClass()
let equalsCss = css({
	[`.${equalsClass}`]: { color: colors.gray },
})

let differenceClass = cssClass()
let differenceCss = css({
	[`.${differenceClass}`]: {
		alignContent: 'center',
		display: 'grid',
		gridAutoFlow: 'column',
	},
})

export function NumberComparisonChart(props: NumberComparisonChartProps) {
	useCss(
		containerCss,
		numberComparisonContainerCss,
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
		<div class={containerClass}>
			<div class={titleClass}>{props.title}</div>
			<div
				class={cx(
					differenceClass,
					difference > 0
						? positiveClass
						: difference < 0
						? negativeClass
						: equalsClass,
				)}
			>
				{props.valueA === null || props.valueB === null
					? 'N/A'
					: difference == 0
					? 'equal'
					: valueFormatter(difference)}
			</div>
			<div class={numberComparisonContainerClass}>
				<div class={valueClass}>
					{props.valueA !== null ? valueFormatter(props.valueA) : 'N/A'}
				</div>
				<div class={valueClass}>
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
