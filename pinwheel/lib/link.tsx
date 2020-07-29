import { ComponentChildren, h } from './react'
import * as CSS from 'csstype'

export type LinkProps = {
	children?: ComponentChildren
	class?: string
	download?: string
	href?: string
	style?: CSS.Properties
}

export function Link(props: LinkProps) {
	return (
		<a
			class={props.class}
			download={props.download}
			href={props.href}
			style={props.style as any}
		>
			{props.children}
		</a>
	)
}
