import { Children, Link, cx, h } from './deps'

type NestedNavProps = {
	children?: Children
}

export function NestedNav(props: NestedNavProps) {
	return <div class="nested-nav">{props.children}</div>
}

type NestedNavSectionProps = {
	children?: Children
}

export function NestedNavSection(props: NestedNavSectionProps) {
	return <div class="nested-nav-section">{props.children}</div>
}

type NestedNavSectionTitleProps = {
	children?: Children
}

export function NestedNavSectionTitle(props: NestedNavSectionTitleProps) {
	return <div class="nested-nav-section-title">{props.children}</div>
}

type NestedNavItemProps = {
	children?: Children
	href: string
	selected?: boolean
}

export function NestedNavItem(props: NestedNavItemProps) {
	return (
		<div class={cx('nested-nav-item', props.selected && 'nested-nav-selected')}>
			<Link href={props.href}>{props.children}</Link>
		</div>
	)
}
