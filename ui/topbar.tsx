import { Button } from './button'
import { Children, JSX, css, h, useCss } from './deps'
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

let topbarWrapperCss = css({
	[`.topbar-wrapper`]: {
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

let topbarLinkCss = css({
	[`.topbar-link`]: {
		color: variables.colors.text,
		cursor: 'pointer',
		textDecoration: 'none',
	},
	[`.topbar-link:hover`]: {
		filter: 'brightness(90%)',
	},
})

let detailsCss = css({
	[`.topbar-details[open] > summary::before`]: {
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
		[`.topbar-details`]: {
			display: 'none',
		},
	},
	[mobile]: {
		[`.topbar-details`]: {
			display: 'block',
		},
	},
	[`.topbar-details[open] .topbar-hamburger-icon`]: {
		color: 'blue',
		display: 'none',
	},
	[`.topbar-details:not([open]) .topbar-x-icon`]: {
		display: 'none',
	},
})

let summaryCss = css({
	[`.topbar-details-summary`]: {
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
	[`.topbar-details-summary::-webkit-details-marker`]: {
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
		<div class="topbar-wrapper" style={wrapperStyle}>
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
							<a class="topbar-link" href={item.href} key={item.title}>
								{item.title}
							</a>
						),
					)}
				</TopbarItemsWrapper>
			)}
			<details class="topbar-details">
				<summary class="topbar-details-summary">
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

let topbarBrandWrapperCss = css({
	[`.topbar-brand-wrapper`]: {
		alignItems: 'center',
		display: 'grid',
		grid: 'auto / auto auto',
		gridGap: '0.5rem',
		height: '2.5rem',
	},
	[`.topbar-brand-wrapper:hover`]: {
		filter: 'brightness(90%)',
	},
})

let topbarBrandImgCss = css({
	[`.topbar-brand-img`]: { height: '2.5rem', width: '2.5rem' },
})

let topbarBrandSvgCss = css({
	[`.topbar-brand-svg`]: { height: '2.5rem', width: '2.5rem' },
})

let topbarBrandTitleCss = css({
	[`.topbar-brand-title`]: {
		fontSize: '1.75rem',
		fontWeight: 'bold',
		userSelect: 'none',
	},
})

export function TopbarBrand(props: TopbarBrandProps) {
	useCss(
		topbarBrandWrapperCss,
		topbarBrandImgCss,
		topbarBrandSvgCss,
		topbarBrandTitleCss,
	)
	let titleStyle = {
		color: props.textColor,
	}
	return (
		<a class="topbar-link" href={props.logoHref ?? '/'}>
			<div class="topbar-brand-wrapper">
				{props.logoImgUrl ? (
					<img class="topbar-brand-img" srcset={`${props.logoImgUrl} 3x`} />
				) : (
					<div class="topbar-brand-svg">{props.logoElement}</div>
				)}
				{props.title && (
					<div class="topbar-brand-title" style={titleStyle}>
						{props.title}
					</div>
				)}
			</div>
		</a>
	)
}

type TopbarItemsWrapperProps = { children?: Children }

let itemsWrapperCss = css({
	[`.topbar-items-wrapper`]: {
		alignItems: 'center',
		display: 'grid',
		gridAutoFlow: 'column',
		gridColumnGap: '2rem',
		userSelect: 'none',
	},
	[mobile]: {
		[`.topbar-items-wrapper`]: {
			display: 'none',
		},
	},
})

function TopbarItemsWrapper(props: TopbarItemsWrapperProps) {
	useCss(itemsWrapperCss)
	return <nav class="topbar-items-wrapper">{props.children}</nav>
}

type HamburgerMenuProps = {
	textColor: string
}

let hamburgerCss = css({
	[`.topbar-hamburger`]: {
		alignContent: 'space-between',
		cursor: 'pointer',
		display: 'grid',
		height: '15px',
		justifySelf: 'end',
		margin: '-1rem',
		padding: '1rem',
		width: '15px',
	},
	[`.topbar-hamburger:hover`]: {
		filter: 'brightness(90%)',
	},
})

function TopbarHamburger(props: HamburgerMenuProps) {
	useCss(hamburgerCss)

	return (
		<div class="topbar-hamburger">
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

let topbarDropdownWrapperCss = css({
	[`.topbar-dropdown-wrapper`]: {
		left: '0',
		padding: '1rem',
		position: 'absolute',
		right: '0',
		top: '4.5rem',
		zIndex: 1,
	},
})

let topbarDropdownItemCss = css({
	[`.topbar-dropdown-item`]: {
		fontSize: '1.25rem',
		padding: '0.5rem 1rem',
	},
	[`.topbar-dropdown-item:hover`]: {
		backgroundColor: variables.colors.hover,
	},
})

let topbarDropdownLinkCss = css({
	[`.topbar-dropdown-link`]: {
		color: variables.colors.text,
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
		<div class="topbar-dropdown-wrapper" style={wrapperStyle}>
			{props.items &&
				props.items.map(item => (
					<a class="topbar-dropdown-link" href={item.href} key={item.title}>
						<div class="topbar-dropdown-item" key={item.title}>
							{item.title}
						</div>
					</a>
				))}
			{props.cta && (
				<div class="topbar-dropdown-item">
					<Button color={variables.colors.accent} href={props.cta.href}>
						{props.cta.title}
					</Button>
				</div>
			)}
		</div>
	)
}
