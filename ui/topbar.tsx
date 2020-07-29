import { Button } from './button'
import { Children, JSX, css, cssClass, h, useCss } from './deps'
import { desktop, mobile, variables } from './theme'

type TopbarProps = {
	activeTextColor: string
	backgroundColor: string
	border?: string
	dropdownBackgroundColor: string
	foregroundColor: string
	items?: TopbarItem[]
	logo?: JSX.Element
	logoHref?: string
	logoImgUrl?: string
	menuSeparatorColor: string
	title?: string
}

export type TopbarItem = {
	element?: JSX.Element
	href: string
	title: string
}

let topbarWrapperClass = cssClass()
let topbarWrapperCss = css({
	[`.${topbarWrapperClass}`]: {
		alignItems: 'center',
		display: 'grid',
		gridAutoFlow: 'column',
		height: '4.5rem',
		justifyContent: 'space-between',
		left: '0',
		padding: '0 1rem',
		position: 'relative',
		right: '0',
		top: '0',
	},
})

let topbarLinkClass = cssClass()
let topbarLinkCss = css({
	[`.${topbarLinkClass}`]: {
		color: variables.colors.text,
		cursor: 'pointer',
		textDecoration: 'none',
	},
	[`.${topbarLinkClass}:hover`]: {
		filter: 'brightness(90%)',
	},
})

let detailsClass = cssClass()
let detailsCss = css({
	[`.${detailsClass}[open] > summary::before`]: {
		background: 'transparent',
		bottom: '0',
		content: '" "',
		cursor: 'default',
		display: 'block',
		left: '0',
		position: 'fixed',
		right: '0',
		top: '0',
		zIndex: 1,
	},
	[desktop]: {
		[`.${detailsClass}`]: {
			display: 'none',
		},
	},
	[mobile]: {
		[`.${detailsClass}`]: {
			display: 'block',
		},
	},
	[`.${detailsClass}[open] .topbar-hamburger-icon`]: {
		color: 'blue',
		display: 'none',
	},
	[`.${detailsClass}:not([open]) .topbar-x-icon`]: {
		display: 'none',
	},
})

let summaryClass = cssClass()
let summaryCss = css({
	[`.${summaryClass}`]: {
		['appearance' as any]: 'none',
		alignItems: 'center',
		boxSizing: 'border-box',
		color: 'inherit',
		cursor: 'pointer',
		display: 'grid',
		font: 'inherit',
		fontSize: '1rem',
		height: '100%',
		outline: 'none',
		padding: '0rem 0.5rem',
		textAlign: 'center',
		userSelect: 'text',
		width: '100%',
	},
	[`.${summaryClass}::-webkit-details-marker`]: {
		display: 'none',
	},
})

export function Topbar(props: TopbarProps) {
	useCss(topbarWrapperCss, topbarLinkCss, detailsCss, summaryCss)
	let wrapperStyle = {
		backgroundColor: props.backgroundColor,
		borderBottom: props.border as any,
		color: props.foregroundColor,
	}
	return (
		<div class={topbarWrapperClass} style={wrapperStyle}>
			<TopbarBrand
				logoElement={props.logo}
				logoHref={props.logoHref}
				logoImgUrl={props.logoImgUrl}
				textColor={props.foregroundColor}
				title={props.title}
			/>
			{props.items && (
				<TopbarItemsWrapper>
					{props.items.map(item =>
						item.element ? (
							item.element
						) : (
							<a class={topbarLinkClass} href={item.href} key={item.title}>
								{item.title}
							</a>
						),
					)}
				</TopbarItemsWrapper>
			)}
			<details class={detailsClass}>
				<summary class={summaryClass}>
					<TopbarHamburger textColor={props.foregroundColor} />
				</summary>
				<TopbarDropdown
					activeTextColor={props.activeTextColor}
					backgroundColor={props.dropdownBackgroundColor}
					border={props.border}
					items={props.items}
					textColor={props.foregroundColor}
				/>
			</details>
		</div>
	)
}

type TopbarBrandProps = {
	logoElement?: JSX.Element
	logoHref?: string
	logoImgUrl?: string
	textColor: string
	title?: string
}

let topbarBrandWrapperClass = cssClass()
let topbarBrandWrapperCss = css({
	[`.${topbarBrandWrapperClass}`]: {
		alignItems: 'center',
		display: 'grid',
		grid: 'auto / auto auto',
		gridGap: '0.5rem',
		height: '2.5rem',
	},
	[`.${topbarBrandWrapperClass}:hover`]: {
		filter: 'brightness(90%)',
	},
})

let topbarBrandImgClass = cssClass()
let topbarBrandImgCss = css({
	[`.${topbarBrandImgClass}`]: { height: '2.5rem', width: '2.5rem' },
})

let topbarBrandSvgClass = cssClass()
let topbarBrandSvgCss = css({
	[`.${topbarBrandSvgClass}`]: { height: '2.5rem', width: '2.5rem' },
})

let topbarBrandTitleClass = cssClass()
let topbarBrandTitleCss = css({
	[`.${topbarBrandTitleClass}`]: {
		fontSize: '1.75rem',
		fontWeight: 'bold',
		userSelect: 'none',
	},
})

export function TopbarBrand(props: TopbarBrandProps) {
	useCss(topbarBrandWrapperCss)
	useCss(topbarBrandImgCss)
	useCss(topbarBrandSvgCss)
	useCss(topbarBrandTitleCss)
	let titleStyle = {
		color: props.textColor,
	}

	return (
		<a class={topbarLinkClass} href={props.logoHref ?? '/'}>
			<div class={topbarBrandWrapperClass}>
				{props.logoImgUrl ? (
					<img class={topbarBrandImgClass} srcset={`${props.logoImgUrl} 3x`} />
				) : (
					<div class={topbarBrandSvgClass}>{props.logoElement}</div>
				)}
				{props.title && (
					<div class={topbarBrandTitleClass} style={titleStyle}>
						{props.title}
					</div>
				)}
			</div>
		</a>
	)
}

type TopbarItemsWrapperProps = { children?: Children }

let itemsWrapperClass = cssClass()
let itemsWrapperCss = css({
	[`.${itemsWrapperClass}`]: {
		alignItems: 'center',
		display: 'grid',
		gridAutoFlow: 'column',
		gridColumnGap: '2rem',
		userSelect: 'none',
	},
	[mobile]: {
		[`.${itemsWrapperClass}`]: {
			display: 'none',
		},
	},
})

function TopbarItemsWrapper(props: TopbarItemsWrapperProps) {
	useCss(itemsWrapperCss)
	return <nav class={itemsWrapperClass}>{props.children}</nav>
}

type HamburgerMenuProps = {
	textColor: string
}

let hamburgerClass = cssClass()
let hamburgerCss = css({
	[`.${hamburgerClass}`]: {
		alignContent: 'space-between',
		cursor: 'pointer',
		display: 'grid',
		height: '15px',
		justifySelf: 'end',
		margin: '-1rem',
		padding: '1rem',
		width: '15px',
	},
	[`.${hamburgerClass}:hover`]: {
		filter: 'brightness(90%)',
	},
})

function TopbarHamburger(props: HamburgerMenuProps) {
	useCss(hamburgerCss)

	return (
		<div class={topbarDropdownWrapperClass}>
			<svg
				class="topbar-hamburger-icon"
				height="15px"
				overflow="visible"
				viewBox="0 0 1 1"
				width="15px"
			>
				{[0, 0.5, 1].map(y => (
					<line
						key={y}
						stroke={props.textColor}
						stroke-linecap="round"
						stroke-width="0.2"
						x1="0"
						x2="1"
						y1={y}
						y2={y}
					/>
				))}
			</svg>
			<svg
				class="topbar-x-icon"
				height="15px"
				overflow="visible"
				viewBox="0 0 1 1"
				width="15px"
			>
				<line
					stroke={props.textColor}
					stroke-linecap="round"
					stroke-width="0.2"
					x1="0"
					x2="1"
					y1="0"
					y2="1"
				/>
				<line
					stroke={props.textColor}
					stroke-linecap="round"
					stroke-width="0.2"
					x1="1"
					x2="0"
					y1="0"
					y2="1"
				/>
			</svg>
		</div>
	)
}

type TopbarMenuProps = {
	activeTextColor: string
	backgroundColor: string
	border?: string
	cta?: TopbarItem
	items?: TopbarItem[]
	textColor: string
}

let topbarDropdownWrapperClass = cssClass()
let topbarDropdownWrapperCss = css({
	[`.${topbarDropdownWrapperClass}`]: {
		left: '0',
		padding: '1rem',
		position: 'absolute',
		right: '0',
		top: '4.5rem',
		zIndex: 1,
	},
})

let topbarDropdownItemClass = cssClass()
let topbarDropdownItemCss = css({
	[`.${topbarDropdownItemClass}`]: {
		fontSize: '1.25rem',
		padding: '0.5rem 1rem',
	},
	[`.${topbarDropdownItemClass}:hover`]: {
		backgroundColor: variables.colors.hover,
	},
})

let topbarDropdownLinkClass = cssClass()
let topbarDropdownLinkCss = css({
	[`.${topbarDropdownLinkClass}`]: {
		cursor: 'pointer',
		textDecoration: 'none',
	},
})

function TopbarDropdown(props: TopbarMenuProps) {
	useCss(topbarDropdownWrapperCss, topbarDropdownItemCss, topbarDropdownLinkCss)
	let wrapperStyle = {
		backgroundColor: props.backgroundColor,
		borderBottom: props.border as any,
	}
	return (
		<div class={topbarDropdownWrapperClass} style={wrapperStyle}>
			{props.items &&
				props.items.map(item => (
					<a class={topbarDropdownLinkClass} href={item.href} key={item.title}>
						<div class={topbarDropdownItemClass} key={item.title}>
							{item.title}
						</div>
					</a>
				))}
			{props.cta && (
				<div class={topbarDropdownItemClass}>
					<Button color={variables.colors.accent} href={props.cta.href}>
						{props.cta.title}
					</Button>
				</div>
			)}
		</div>
	)
}
