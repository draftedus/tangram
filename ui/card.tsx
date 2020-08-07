import { Children, h } from './deps'

export type CardProps = {
	children?: Children
}

export function Card(props: CardProps) {
	return <div class="card">{props.children}</div>
}
