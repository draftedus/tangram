import "./details.css"
import { h } from "preact"

export type DetailsProps = {
	options?: DetailsOption[]
	summary: string | null
}

export type DetailsOption = {
	href: string
	title: string
}

export function Details(props: DetailsProps) {
	return (
		<details class="details">
			<summary class="details-summary" role="button">
				{props.summary}
			</summary>
			<div class="details-list">
				{props.options?.map(option => (
					<a class="details-list-item" href={option.href} key={option.title}>
						{option.title}
					</a>
				))}
			</div>
		</details>
	)
}
