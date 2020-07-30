import { css, cssClass, h, useCss } from './deps'
import { mobile, variables } from './theme'

type NumberChartProps = {
	title: string
	value: string
}

let containerClass = cssClass()
let containerCss = css({
	[`.${containerClass}`]: {
		color: variables.colors.text,
		display: 'flex',
		flexDirection: 'column',
		justifyContent: 'center',
		textAlign: 'center',
	},
})

let valueClass = cssClass()
let valueCss = css({
	[`.${valueClass}`]: {
		fontSize: '2rem',
		marginBottom: '1rem',
	},
	[mobile]: {
		[`.${valueClass}`]: {
			fontSize: '1.5rem',
		},
	},
})
let titleClass = cssClass()
let titleCss = css({
	[`.${titleClass}`]: { color: variables.colors.text },
})

export function NumberChart(props: NumberChartProps) {
	useCss(containerCss, valueCss, titleCss)
	return (
		<div class={containerClass}>
			<div class={valueClass}>{props.value}</div>
			<div class={titleClass}>{props.title}</div>
		</div>
	)
}
