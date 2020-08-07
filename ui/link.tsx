import { Children, Link as PinwheelLink, cx, h } from './deps'

export type LinkProps = {
	children?: Children
	className?: string
	href?: string
	title?: string
}

export function Link(props: LinkProps) {
	let className = cx('link', props.className)
	return (
		<PinwheelLink class={className} href={props.href} title={props.title}>
			{props.children}
		</PinwheelLink>
	)
}
