import { Children, Link, cx, h } from './deps'

type TabBarProps = { children?: Children }

export function TabBar(props: TabBarProps) {
	return <div class="tab-bar">{props.children}</div>
}

type TabProps = {
	children?: Children
	disabled?: boolean
	onClick?: () => void
	selected?: boolean
}

export function Tab(props: TabProps) {
	return (
		<div
			class={cx(
				'tab-bar-tab',
				props.selected && 'tab-bar-tab-selected',
				props.disabled && 'tab-bar-tab-disbaled',
			)}
			onClick={!props.disabled ? props.onClick : undefined}
		>
			{props.children}
		</div>
	)
}

type TabLinkProps = {
	children?: Children
	disabled?: boolean
	href: string
}

export function TabLink(props: TabLinkProps) {
	let selected = false
	return (
		<Tab selected={selected}>
			<Link class="tab-bar-tab-link" href={props.href}>
				{props.children}
			</Link>
		</Tab>
	)
}
