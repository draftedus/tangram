import './button.css'
import { ComponentChildren, h } from 'preact'

export type ButtonProps = {
	block?: boolean
	children?: ComponentChildren
	color?: string
	disabled?: boolean
	download?: string
	href?: string
	id?: string
	onClick?: () => void
	type?: 'submit' | 'button' | 'reset'
}

export function Button(props: ButtonProps) {
	let style = {
		backgroundColor: props.color as any,
	}
	let onClick = !props.disabled && props.onClick ? props.onClick : undefined
	if (props.href) {
		return (
			<a
				class="button"
				download={props.download}
				href={props.href}
				style={style}
			>
				{props.children}
			</a>
		)
	} else {
		return (
			<button
				class="button"
				id={props.id}
				onClick={onClick}
				style={style}
				type={props.type}
			>
				{props.children}
			</button>
		)
	}
}
