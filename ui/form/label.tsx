import { Children, css, cssClass, h, useCss } from '../deps'
import { variables } from '../theme'

type Props = {
	children?: Children
	for?: string
}

let labelClass = cssClass()
let labelCss = css({
	[`.${labelClass}`]: {
		color: variables.colors.text,
		display: 'grid',
		gridGap: '0.5rem',
		userSelect: 'none',
	},
})

export function Label(props: Props) {
	useCss(labelCss)
	return (
		<label class={labelClass} htmlFor={props.for}>
			{props.children}
		</label>
	)
}
