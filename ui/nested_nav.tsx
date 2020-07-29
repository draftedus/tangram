import { Children, Link, css, cssClass, cx, h, useCss } from './deps'
import { variables } from './theme'

type NestedNavProps = {
	children?: Children
}

let nestedNavClass = cssClass()
let nestedNavCss = css({
	[`.${nestedNavClass}`]: { display: 'grid', gap: '1.5rem' },
})

export function NestedNav(props: NestedNavProps) {
	useCss(nestedNavCss)
	return <div class={nestedNavClass}>{props.children}</div>
}

type NestedNavSectionProps = {
	children?: Children
}

let nestedNavSectionClass = cssClass()
let nestedNavSectionCss = css({
	[`.${nestedNavSectionClass}`]: { display: 'grid', gap: '0.75rem' },
})

export function NestedNavSection(props: NestedNavSectionProps) {
	useCss(nestedNavSectionCss)
	return <div class={nestedNavSectionClass}>{props.children}</div>
}

type NestedNavSectionTitleProps = {
	children?: Children
}

let nestedNavSectionTitleClass = cssClass()
let nestedNavSectionTitleCss = css({
	[`.${nestedNavSectionTitleClass}`]: { fontSize: '1rem', fontWeight: 'bold' },
})

export function NestedNavSectionTitle(props: NestedNavSectionTitleProps) {
	useCss(nestedNavSectionTitleCss)
	return <div class={nestedNavSectionTitleClass}>{props.children}</div>
}

type NestedNavItemProps = {
	children?: Children
	highlight?: boolean
	href: string
}

let nestedNavItemClass = cssClass()
let nestedNavItemCss = css({
	[`.${nestedNavItemClass}`]: {
		borderLeft: `${variables.border.width} solid ${variables.colors.border}`,
		display: 'grid',
		gap: '0.5rem',
		outline: 'none',
		overflow: 'hidden',
		paddingLeft: '1rem',
		textOverflow: 'ellipsis',
		userSelect: 'none',
	},
	[`.${nestedNavItemClass} > a`]: {
		color: variables.colors.mutedText,
		cursor: 'pointer',
		textDecoration: 'none',
	},
})

let nestedNavHighlightClass = cssClass()
let nestedNavHighlightCss = css({
	[`.${nestedNavHighlightClass}`]: {
		borderLeftColor: variables.colors.accent,
	},
	[`.${nestedNavHighlightClass} > a`]: {
		color: variables.colors.accent,
	},
	[`.${nestedNavHighlightClass} > a:hover`]: {
		filter: 'brightness(90%)',
	},
})

export function NestedNavItem(props: NestedNavItemProps) {
	useCss(nestedNavItemCss, nestedNavHighlightCss)
	return (
		<div
			class={cx(nestedNavItemClass, props.highlight && nestedNavHighlightClass)}
		>
			<Link href={props.href}>{props.children}</Link>
		</div>
	)
}
