import "./link.css"
import { cx } from "./util"
import { ComponentChildren, h } from "preact"

export type LinkProps = {
	children?: ComponentChildren
	className?: string
	href?: string
	title?: string
}

export function Link(props: LinkProps) {
	let className = cx("link", props.className)
	return (
		<a class={className} href={props.href} title={props.title}>
			{props.children}
		</a>
	)
}
