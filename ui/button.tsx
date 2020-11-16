import "./button.css"
import { ComponentChildren, h } from "preact"

export type ButtonProps = {
	block?: boolean
	children?: ComponentChildren
	color?: string
	disabled?: boolean
	download?: string
	href?: string
	id?: string
	onClick?: () => void
	type?: "submit" | "button" | "reset"
}

export function Button(props: ButtonProps) {
	let style = {
		backgroundColor: props.color,
	}
	let onClick = !props.disabled && props.onClick ? props.onClick : undefined
	if (props.href) {
		return (
			<a
				class="button"
				disabled={props.disabled}
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
				disabled={props.disabled}
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
