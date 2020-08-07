import { Children, Link, h } from './deps'

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

export function Button(props: ButtonProps) {
	let style = {
		backgroundColor: props.color as any,
	}
	let onClick = !props.disabled && props.onClick ? props.onClick : undefined
	if (props.href) {
		return (
			<Link
				class="button"
				download={props.download}
				href={props.href}
				style={style}
			>
				{props.children}
			</Link>
		)
	} else {
		return (
			<button class="button" onClick={onClick} style={style} type={props.type}>
				{props.children}
			</button>
		)
	}
}
