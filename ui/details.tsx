import { css, h, useCss } from './deps'
import { border, variables } from './theme'

export type DetailsSelectProps = {
	options?: DetailsSelectOption[]
	summary: string | null
}

export type DetailsSelectOption = {
	href: string
	name: string
}

let detailsCss = css({
	[`.details`]: {
		boxSizing: 'border-box',
		position: 'relative',
	},
	[`.details[open] > summary`]: {
		borderColor: variables.colors.accent,
	},
	[`.details[open] > summary::before`]: {
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

let summaryCss = css({
	[`.details-summary`]: {
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
	[`.details-summary:hover`]: {
		borderColor: variables.colors.hover,
	},
	[`.details-summary::-webkit-details-marker`]: {
		display: 'none',
	},
})

let detailsListCss = css({
	[`.details-list`]: {
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

let detailsListItemCss = css({
	[`.details-list-item`]: {
		color: variables.colors.text,
		display: 'block',
		listStyle: 'none',
		padding: '0.5rem',
		textDecoration: 'none',
	},
	[`.details-list-item:hover`]: {
		backgroundColor: variables.colors.hover,
	},
})

export function Details(props: DetailsSelectProps) {
	useCss(detailsCss, summaryCss, detailsListCss, detailsListItemCss)

	return (
		<details class="details">
			<summary class="details-summary" role="button">
				{props.summary}
			</summary>
			<div class="details-list">
				{props.options?.map(option => (
					<a class="details-list-item" href={option.href} key={option.name}>
						{option.name}
					</a>
				))}
			</div>
		</details>
	)
}
