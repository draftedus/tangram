import { Children, css, cssClass, h, useCss } from './deps'
import { border, variables } from './theme'

export type CardProps = {
	children?: Children
}

let cardClass = cssClass()
let cardCss = css({
	[`.${cardClass}`]: {
		backgroundColor: variables.colors.surface,
		border,
		borderRadius: variables.border.radius,
		padding: '1rem',
	},
})

export function Card(props: CardProps) {
	useCss(cardCss)
	return <div class={cardClass}>{props.children}</div>
}
