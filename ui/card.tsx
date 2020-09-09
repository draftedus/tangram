import { ComponentChildren, h } from 'preact'

export type CardProps = {
	children?: ComponentChildren
}

export function Card(props: CardProps) {
	return <div class="card">{props.children}</div>
}
