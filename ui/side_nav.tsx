import { Children, Link, cx, h } from './deps'

type SideNavProps = { children?: Children }

export function SideNav(props: SideNavProps) {
	return <div class="side-nav">{props.children}</div>
}

type SideNavSectionProps = { children?: Children }

export function SideNavSection(props: SideNavSectionProps) {
	return <div class="side-nav-section">{props.children}</div>
}

type SideNavTitleProps = { children?: Children }

export function SideNavTitle(props: SideNavTitleProps) {
	return <div class="side-nav-title">{props.children}</div>
}

type SideNavItemProps = {
	children?: Children
	href: string
	selected?: boolean
}

export function SideNavItem(props: SideNavItemProps) {
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
