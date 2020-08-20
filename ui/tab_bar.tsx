import { Children, cx, h } from './deps'

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
	let className = cx(
		'tab-bar-tab',
		props.selected && 'tab-bar-tab-selected',
		props.disabled && 'tab-bar-tab-disbaled',
	)
	return (
		<div
			class={className}
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
	selected?: boolean
}

export function TabLink(props: TabLinkProps) {
	return (
		<Tab selected={props.selected}>
			<a class="tab-bar-tab-link" href={props.href}>
				{props.children}
			</a>
		</Tab>
	)
}
