import {
	Children,
	Link as PinwheelLink,
	css,
	cssClass,
	cx,
	h,
	useCss,
} from './deps'
import { colors } from './theme'

export type LinkProps = {
	children?: Children
	className?: string
	href?: string
}

let linkClass = cssClass()
let linkCss = css({
	[`.${linkClass}`]: {
		color: colors.blue,
		cursor: 'pointer',
		textDecoration: 'none',
	},
	[`.${linkClass}:hover`]: {
		filter: 'brightness(90%)',
	},
})

export function Link(props: LinkProps) {
	useCss(linkCss)
	let className = cx(props.className, linkClass)
	return (
		<PinwheelLink class={className} href={props.href}>
			{props.children}
		</PinwheelLink>
	)
}
