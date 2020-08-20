import { Children, cx, h } from './deps'

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
	let className = cx(
		'side-nav-item',
		props.selected && 'side-nav-item-selected',
	)
	return (
		<a class={className} href={props.href}>
			{props.children}
		</a>
	)
}
