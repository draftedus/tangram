import { Children, Link as PinwheelLink, css, cx, h, useCss } from './deps'
import { colors } from './theme'

export type LinkProps = {
	children?: Children
	className?: string
	href?: string
	title?: string
}

let linkCss = css({
	[`.link`]: {
		color: colors.blue,
		cursor: 'pointer',
		textDecoration: 'none',
	},
	[`.link:hover`]: {
		filter: 'brightness(90%)',
	},
	[`.link:focus`]: {
		filter: 'brightness(90%)',
	},
})

export function Link(props: LinkProps) {
	useCss(linkCss)
	let className = cx(props.className, 'link')
	return (
		<PinwheelLink class={className} href={props.href} title={props.title}>
			{props.children}
		</PinwheelLink>
	)
}
