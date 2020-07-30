import { Children, Link, css, cssClass, cx, h, useCss } from './deps'
import { variables } from './theme'

type SideNavProps = { children?: Children }

let sideNavClass = cssClass()
let sideNavCss = css({
	[`.${sideNavClass}`]: {
		alignContent: 'start',
		display: 'grid',
		gridGap: '1rem',
		height: '100%',
	},
})

export function SideNav(props: SideNavProps) {
	useCss(sideNavCss)
	return <div class={sideNavClass}>{props.children}</div>
}

type SideNavSectionProps = { children?: Children }

let sectionClass = cssClass()
let sectionCss = css({
	[`.${sectionClass}`]: { display: 'grid' },
})

export function SideNavSection(props: SideNavSectionProps) {
	useCss(sectionCss)
	return <div class={sectionClass}>{props.children}</div>
}

type SideNavTitleProps = { children?: Children }

let titleClass = cssClass()
let titleCss = css({
	[`.${titleClass}`]: {
		fontSize: '1rem',
		fontWeight: 'bold',
		paddingTop: '1rem',
	},
})

export function SideNavTitle(props: SideNavTitleProps) {
	useCss(titleCss)
	return <div class={titleClass}>{props.children}</div>
}

type SideNavItemProps = {
	children?: Children
	href: string
	selected?: boolean
}

let itemClass = cssClass()
let itemCss = css({
	[`.${itemClass}`]: {
		color: variables.colors.text,
		cursor: 'pointer',
		filter: 'none',
		paddingBottom: '.5rem',
		paddingTop: '.5rem',
		textDecoration: 'none',
	},
})

let selectedClass = cssClass()
let selectedCss = css({
	[`.${selectedClass}`]: {
		color: variables.colors.accent,
	},
	[`.${selectedClass}:hover`]: {
		filter: 'brightness(90%)',
	},
})

export function SideNavItem(props: SideNavItemProps) {
	useCss(itemCss, selectedCss)
	let selected = false
	return (
		<Link
			class={cx(selectedClass, selected && selectedClass)}
			href={props.href}
		>
			{props.children}
		</Link>
	)
}
