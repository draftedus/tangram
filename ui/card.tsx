import { ComponentChildren, h } from './deps'

export type CardProps = {
	children?: ComponentChildren
}

export function Card(props: CardProps) {
	return <div class="card">{props.children}</div>
}
