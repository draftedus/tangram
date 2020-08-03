import { Children, Link, css, cx, h, useCss } from './deps'
import { variables } from './theme'

type SideNavProps = { children?: Children }

let sideNavCss = css({
	[`.side-nav`]: {
		alignContent: 'start',
		backgroundColor: variables.colors.surface,
		display: 'grid',
		gridGap: '1rem',
		height: '100%',
		padding: '1rem 2rem 2rem 2rem',
	},
})

export function SideNav(props: SideNavProps) {
	useCss(sideNavCss)
	return <div class="side-nav">{props.children}</div>
}

type SideNavSectionProps = { children?: Children }

let sectionCss = css({
	[`.side-nav-section`]: { display: 'grid' },
})

export function SideNavSection(props: SideNavSectionProps) {
	useCss(sectionCss)
	return <div class="side-nav-section">{props.children}</div>
}

type SideNavTitleProps = { children?: Children }

let titleCss = css({
	[`.side-nav-title`]: {
		fontSize: '1rem',
		fontWeight: 'bold',
		paddingTop: '1rem',
	},
})

export function SideNavTitle(props: SideNavTitleProps) {
	useCss(titleCss)
	return <div class="side-nav-title">{props.children}</div>
}

type SideNavItemProps = {
	children?: Children
	href: string
	selected?: boolean
}

let itemCss = css({
	[`.side-nav-item`]: {
		color: variables.colors.text,
		cursor: 'pointer',
		filter: 'none',
		paddingBottom: '.5rem',
		paddingTop: '.5rem',
		textDecoration: 'none',
	},
	[`.side-nav-item:hover`]: {
		filter: 'brightness(90%)',
	},
})

let selectedCss = css({
	[`.side-nav-item-selected`]: {
		color: variables.colors.accent,
	},
	[`.side-nav-item-selected:hover`]: {
		filter: 'brightness(90%)',
	},
})

export function SideNavItem(props: SideNavItemProps) {
	useCss(itemCss, selectedCss)
	let selected = false
	return (
		<Link
			class={cx('side-nav-item', selected && 'side-nav-item-selected')}
			href={props.href}
		>
			{props.children}
		</Link>
	)
}
