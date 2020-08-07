import { Children, h } from './deps'
import { variables } from './theme'

export type TokenProps = {
	children?: Children
	color?: string
	inline?: boolean
	textColor?: string
}

export function Token(props: TokenProps) {
	let style = {
		backgroundColor: props.color ?? variables.colors.accent,
		color: props.textColor ?? variables.colors.funText,
	}

	return (
		<div class="token" style={style}>
			{props.children}
		</div>
	)
}
