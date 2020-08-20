import { Children, cx, h } from './deps'

export type LinkProps = {
	children?: Children
	className?: string
	href?: string
	title?: string
}

export function Link(props: LinkProps) {
	let className = cx('link', props.className)
	return (
		<a class={className} href={props.href} title={props.title}>
			{props.children}
		</a>
	)
}
