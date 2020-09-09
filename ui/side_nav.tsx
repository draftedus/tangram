import { cx } from '@tangramhq/pinwheel'
import { ComponentChildren, h } from 'preact'

type SideNavProps = { children?: ComponentChildren }

export function SideNav(props: SideNavProps) {
	return <div class="side-nav">{props.children}</div>
}

type SideNavSectionProps = { children?: ComponentChildren }

export function SideNavSection(props: SideNavSectionProps) {
	return <div class="side-nav-section">{props.children}</div>
}

type SideNavTitleProps = { children?: ComponentChildren }

export function SideNavTitle(props: SideNavTitleProps) {
	return <div class="side-nav-title">{props.children}</div>
}

type SideNavItemProps = {
	children?: ComponentChildren
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
