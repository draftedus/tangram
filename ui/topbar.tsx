import { Button } from "./button"
import "./topbar.css"
import { ComponentChildren, h } from "preact"

type TopbarProps = {
	backgroundColor: string
	border?: string
	dropdownBackgroundColor: string
	foregroundColor: string
	items?: TopbarItem[]
	logo?: ComponentChildren
	logoHref?: string
	logoImgUrl?: string
	title?: string
}

export type TopbarItem = {
	element?: ComponentChildren
	href: string
	title: string
}

export function Topbar(props: TopbarProps) {
	let wrapperStyle = {
		backgroundColor: props.backgroundColor,
		borderBottom: props.border,
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
					backgroundColor={props.dropdownBackgroundColor}
					border={props.border}
					items={props.items}
				/>
			</details>
		</div>
	)
}

type TopbarBrandProps = {
	logoElement?: ComponentChildren
	logoHref?: string
	logoImgUrl?: string
	textColor: string
	title?: string
}

export function TopbarBrand(props: TopbarBrandProps) {
	let titleStyle = {
		color: props.textColor,
	}
	return (
		<a class="topbar-link" href={props.logoHref ?? "/"}>
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

type TopbarItemsWrapperProps = { children?: ComponentChildren }

function TopbarItemsWrapper(props: TopbarItemsWrapperProps) {
	return <nav class="topbar-items-wrapper">{props.children}</nav>
}

type HamburgerMenuProps = {
	textColor: string
}

function TopbarHamburger(props: HamburgerMenuProps) {
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
	backgroundColor: string
	border?: string
	cta?: TopbarItem
	items?: TopbarItem[]
}

function TopbarDropdown(props: TopbarMenuProps) {
	let wrapperStyle = {
		backgroundColor: props.backgroundColor,
		borderBottom: props.border,
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
					<Button href={props.cta.href}>{props.cta.title}</Button>
				</div>
			)}
		</div>
	)
}
