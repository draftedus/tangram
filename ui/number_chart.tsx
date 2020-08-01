import { css, h, useCss } from './deps'
import { mobile, variables } from './theme'

type NumberChartProps = {
	title: string
	value: string
}

let wrapperCss = css({
	[`.number-chart-wrapper`]: {
		color: variables.colors.text,
		display: 'flex',
		flexDirection: 'column',
		justifyContent: 'center',
		textAlign: 'center',
	},
})

let valueCss = css({
	[`.number-chart-value`]: {
		fontSize: '2rem',
		marginBottom: '1rem',
	},
	[mobile]: {
		[`.number-chart-value`]: {
			fontSize: '1.5rem',
		},
	},
})
let titleCss = css({
	[`.number-chart-title`]: { color: variables.colors.text },
})

export function NumberChart(props: NumberChartProps) {
	useCss(wrapperCss, valueCss, titleCss)
	return (
		<div class="number-chart-wrapper">
			<div class="number-chart-value">{props.value}</div>
			<div class="number-chart-title">{props.title}</div>
		</div>
	)
}
