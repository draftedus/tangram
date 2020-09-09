import { ComponentChildren, cx, h } from './deps'

type NestedNavProps = {
	children?: ComponentChildren
}

export function NestedNav(props: NestedNavProps) {
	return <div class="nested-nav">{props.children}</div>
}

type NestedNavSectionProps = {
	children?: ComponentChildren
}

export function NestedNavSection(props: NestedNavSectionProps) {
	return <div class="nested-nav-section">{props.children}</div>
}

type NestedNavSectionTitleProps = {
	children?: ComponentChildren
}

export function NestedNavSectionTitle(props: NestedNavSectionTitleProps) {
	return <div class="nested-nav-section-title">{props.children}</div>
}

type NestedNavItemProps = {
	children?: ComponentChildren
	href: string
	selected?: boolean
}

export function NestedNavItem(props: NestedNavItemProps) {
	return (
		<div class={cx('nested-nav-item', props.selected && 'nested-nav-selected')}>
			<a href={props.href}>{props.children}</a>
		</div>
	)
}
