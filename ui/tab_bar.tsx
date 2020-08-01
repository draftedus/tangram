import { Children, Link, css, cx, h, useCss } from './deps'
import { border, variables } from './theme'

type TabBarProps = { children?: Children }

let tabBarCss = css({
	[`.tab-bar`]: {
		borderBottom: border,
		display: 'grid',
		gridAutoFlow: 'column',
		gridColumnGap: '2rem',
		justifyContent: 'start',
	},
})

export function TabBar(props: TabBarProps) {
	useCss(tabBarCss)
	return <div class="tab-bar">{props.children}</div>
}

type TabProps = {
	children?: Children
	disabled?: boolean
	onClick?: () => void
	selected?: boolean
}

let tabCss = css({
	[`.tab-bar-tab`]: {
		borderBottom: `${variables.border.width} solid ${variables.colors.border}`,
		color: variables.colors.mutedText,
		cursor: 'pointer',
		height: '1.5rem',
		marginBottom: `-${variables.border.width}`,
		position: 'relative',
		userSelect: 'none',
	},
	[`.tab-bar-tab:hover`]: {
		filter: `brightness(90%)`,
	},
})

let selectedTabCss = css({
	[`.tab-bar-tab-selected`]: {
		borderBottomColor: variables.colors.accent,
		color: variables.colors.accent,
	},
	[`.tab-bar-tab-selected:hover`]: {
		filter: 'none',
	},
})

let disabledTabCss = css({
	[`.tab-bar-tab-disabled`]: { cursor: 'not-allowed' },
})

export function Tab(props: TabProps) {
	useCss(tabCss, selectedTabCss, disabledTabCss)
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

let tabLinkCss = css({
	[`.tab-bar-tab-link`]: {
		color: 'inherit',
		display: 'block',
		height: '100%',
		textDecoration: 'none',
	},
})

export function TabLink(props: TabLinkProps) {
	let selected = false
	useCss(tabLinkCss)
	return (
		<Tab selected={selected}>
			<Link class="tab-bar-tab-link" href={props.href}>
				{props.children}
			</Link>
		</Tab>
	)
}
