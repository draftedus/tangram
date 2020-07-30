import { Children, Link, css, cssClass, cx, h, useCss } from './deps'
import { border, variables } from './theme'

type TabBarProps = { children?: Children }

let tabBarClass = cssClass()
let tabBarCss = css({
	[`.${tabBarClass}`]: {
		borderBottom: border,
		display: 'grid',
		gridAutoFlow: 'column',
		gridColumnGap: '2rem',
		justifyContent: 'start',
	},
})

export function TabBar(props: TabBarProps) {
	useCss(tabBarCss)
	return <div class={tabBarClass}>{props.children}</div>
}

type TabProps = {
	children?: Children
	disabled?: boolean
	onClick?: () => void
	selected?: boolean
}

let tabClass = cssClass()
let tabCss = css({
	[`.${tabClass}`]: {
		borderBottom: `${variables.border.width} solid ${variables.colors.border}`,
		color: variables.colors.mutedText,
		cursor: 'pointer',
		height: '1.5rem',
		marginBottom: `-${variables.border.width}`,
		position: 'relative',
		userSelect: 'none',
	},
	[`.${tabClass}:hover`]: {
		filter: `brightness(90%)`,
	},
})

let selectedTabClass = cssClass()
let selectedTabCss = css({
	[`.${selectedTabClass}`]: {
		borderBottomColor: variables.colors.accent,
		color: variables.colors.accent,
	},
	[`.${selectedTabClass}:hover`]: {
		filter: 'none',
	},
})

let disabledTabClass = cssClass()
let disabledTabCss = css({
	[`.${disabledTabClass}`]: { cursor: 'not-allowed' },
})

export function Tab(props: TabProps) {
	useCss(tabCss, selectedTabCss, disabledTabCss)
	return (
		<div
			class={cx(
				tabClass,
				props.selected && selectedTabClass,
				props.disabled && disabledTabClass,
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

let tabLinkClass = cssClass()
let tabLinkCss = css({
	[`.${tabLinkClass}`]: {
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
			<Link class={tabLinkClass} href={props.href}>
				{props.children}
			</Link>
		</Tab>
	)
}
