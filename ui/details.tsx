import { css, cssClass, h, useCss } from './deps'
import { border, variables } from './theme'

export type DetailsSelectProps = {
	options?: DetailsSelectOption[]
	summary: string | null
}

export type DetailsSelectOption = {
	href: string
	name: string
}

let detailsClass = cssClass()
let detailsCss = css({
	[`.${detailsClass}`]: {
		boxSizing: 'border-box',
		position: 'relative',
	},
	[`.${detailsClass}[open] > summary`]: {
		borderColor: variables.colors.accent,
	},
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
})

let summaryClass = cssClass()
let summaryCss = css({
	[`.${summaryClass}`]: {
		WebkitTextFillColor: 'inherit',
		alignItems: 'center',
		appearance: 'none',
		backgroundColor: variables.colors.surface,
		border,
		borderRadius: variables.border.radius,
		boxSizing: 'border-box',
		color: 'inherit',
		cursor: 'pointer',
		display: 'grid',
		font: 'inherit',
		fontSize: '1rem',
		height: '2.5rem',
		letterSpacing: 'normal',
		outline: 'none',
		padding: '0.5rem 1rem',
		position: 'relative',
		userSelect: 'text',
		width: '100%',
	},
	[`.${summaryClass}:hover`]: {
		borderColor: variables.colors.hover,
	},
	[`.${summaryClass}::-webkit-details-marker`]: {
		display: 'none',
	},
})

let detailsListClass = cssClass()
let detailsListCss = css({
	[`.${detailsListClass}`]: {
		backgroundColor: variables.colors.surface,
		boxSizing: 'border-box',
		fontSize: '1rem',
		marginTop: '2.5rem',
		padding: '0rem',
		position: 'absolute',
		top: '0',
		width: '100%',
		zIndex: 1,
	},
})

let detailsListItemClass = cssClass()
let detailsListItemCss = css({
	[`.${detailsListItemClass}`]: {
		color: variables.colors.text,
		display: 'block',
		listStyle: 'none',
		padding: '0.5rem',
		textDecoration: 'none',
	},
	[`.${detailsListItemClass}:hover`]: {
		backgroundColor: variables.colors.hover,
	},
})

export function Details(props: DetailsSelectProps) {
	useCss(detailsCss, summaryCss, detailsListCss, detailsListItemCss)

	return (
		<details class={detailsClass}>
			<summary class={summaryClass} role="button">
				{props.summary}
			</summary>
			<div class={detailsListClass}>
				{props.options?.map(option => (
					<a class={detailsListItemClass} href={option.href} key={option.name}>
						{option.name}
					</a>
				))}
			</div>
		</details>
	)
}
