import { Children, css, h, useCss } from './deps'
import { border, variables } from './theme'

export type CardProps = {
	children?: Children
}

let cardCss = css({
	[`.card`]: {
		backgroundColor: variables.colors.surface,
		border,
		borderRadius: variables.border.radius,
		padding: '1rem',
	},
})

export function Card(props: CardProps) {
	useCss(cardCss)
	return <div class="card">{props.children}</div>
}
