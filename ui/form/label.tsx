import { Children, css, cssClass, h, useCss } from '../deps'
import { variables } from '../theme'

type Props = {
	children?: Children
	for?: string
}

let childClass = cssClass()
let labelClass = cssClass()
let labelCss = css({
	[`.${labelClass}`]: {
		color: variables.colors.text,
		display: 'grid',
		fontSize: '.8rem',
		gridGap: '0.5rem',
		letterSpacing: '.1rem',
		userSelect: 'none',
	},
	[`.${labelClass} > .${childClass}`]: {
		color: 'green',
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
