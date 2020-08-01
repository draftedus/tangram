import { Children, Link, css, cx, h, useCss } from './deps'
import { variables } from './theme'

type NestedNavProps = {
	children?: Children
}

let nestedNavCss = css({
	[`.nested-nav`]: { display: 'grid', gap: '1.5rem' },
})

export function NestedNav(props: NestedNavProps) {
	useCss(nestedNavCss)
	return <div class="nested-nav">{props.children}</div>
}

type NestedNavSectionProps = {
	children?: Children
}

let nestedNavSectionCss = css({
	[`.nested-nav-section`]: { display: 'grid', gap: '0.75rem' },
})

export function NestedNavSection(props: NestedNavSectionProps) {
	useCss(nestedNavSectionCss)
	return <div class="nested-nav-section">{props.children}</div>
}

type NestedNavSectionTitleProps = {
	children?: Children
}

let nestedNavSectionTitleCss = css({
	[`.nested-nav-section-title`]: { fontSize: '1rem', fontWeight: 'bold' },
})

export function NestedNavSectionTitle(props: NestedNavSectionTitleProps) {
	useCss(nestedNavSectionTitleCss)
	return <div class="nested-nav-section-title">{props.children}</div>
}

type NestedNavItemProps = {
	children?: Children
	highlight?: boolean
	href: string
}

let nestedNavItemCss = css({
	[`.nested-nav-item`]: {
		borderLeft: `${variables.border.width} solid ${variables.colors.border}`,
		display: 'grid',
		gap: '0.5rem',
		outline: 'none',
		overflow: 'hidden',
		paddingLeft: '1rem',
		textOverflow: 'ellipsis',
		userSelect: 'none',
	},
	[`.nested-nav-item > a`]: {
		color: variables.colors.mutedText,
		cursor: 'pointer',
		textDecoration: 'none',
	},
})

let nestedNavHighlightCss = css({
	[`.nested-nav-highlight`]: {
		borderLeftColor: variables.colors.accent,
	},
	[`.nested-nav-highlight > a`]: {
		color: variables.colors.accent,
	},
	[`.nested-nav-highlight > a:hover`]: {
		filter: 'brightness(90%)',
	},
})

export function NestedNavItem(props: NestedNavItemProps) {
	useCss(nestedNavItemCss, nestedNavHighlightCss)
	return (
		<div
			class={cx('nested-nav-item', props.highlight && 'nested-nav-highlight')}
		>
			<Link href={props.href}>{props.children}</Link>
		</div>
	)
}
