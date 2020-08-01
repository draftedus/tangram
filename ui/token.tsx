import { Children, css, h, useCss } from './deps'
import { variables } from './theme'

export type TokenProps = {
	children?: Children
	color?: string
	inline?: boolean
	textColor?: string
}

let tokenCss = css({
	[`.token`]: {
		alignItems: 'center',
		borderRadius: '4px',
		boxSizing: 'border-box',
		display: 'inline-flex',
		fontSize: '1rem',
		height: '1.5rem',
		padding: '0 0.5rem',
		whiteSpace: 'nowrap',
	},
})

export function Token(props: TokenProps) {
	useCss(tokenCss)
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
