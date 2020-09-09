import { h } from 'preact'

export type DetailsSelectProps = {
	options?: DetailsSelectOption[]
	summary: string | null
}

export type DetailsSelectOption = {
	href: string
	name: string
}

export function Details(props: DetailsSelectProps) {
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
