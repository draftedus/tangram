import { Children, Link, css, cssClass, h, useCss } from './deps'
import { variables } from './theme'

export type ButtonProps = {
	block?: boolean
	children?: Children
	color?: string
	disabled?: boolean
	download?: string
	href?: string
	onClick?: () => void
	type?: 'submit' | 'button'
}

let buttonClass = cssClass()
let buttonCss = css({
	[`.${buttonClass}`]: {
		appearance: 'none',
		backgroundColor: variables.colors.accent,
		border: 'none',
		borderRadius: variables.border.radius,
		boxSizing: 'border-box',
		color: variables.colors.funText,
		cursor: 'pointer',
		display: 'block',
		fontSize: '1rem',
		lineHeight: '1',
		margin: '0',
		outline: 'none',
		overflow: 'hidden',
		padding: '0.75rem 1rem',
		textAlign: 'center',
		textDecoration: 'none',
		textOverflow: 'ellipsis',
		userSelect: 'none',
		whiteSpace: 'nowrap',
		width: '100%',
	},
	[`.${buttonClass}:hover`]: {
		filter: 'brightness(90%)',
	},
	[`.${buttonClass}:active`]: {
		filter: 'brightness(90%)',
	},
	[`.${buttonClass}::-moz-focus-inner`]: {
		border: 'none',
	},
})

export function Button(props: ButtonProps) {
	useCss(buttonCss)
	let style = {
		backgroundColor: props.color as any,
	}
	let onClick = !props.disabled && props.onClick ? props.onClick : undefined
	if (props.href) {
		return (
			<Link
				class={buttonClass}
				download={props.download}
				href={props.href}
				style={style}
			>
				{props.children}
			</Link>
		)
	} else {
		return (
			<button
				class={buttonClass}
				onClick={onClick}
				style={style}
				type={props.type}
			>
				{props.children}
			</button>
		)
	}
}
