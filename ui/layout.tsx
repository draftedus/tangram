import { Children, cx, h } from './deps'

type Props = { children?: Children }

export function S1(props: Props) {
	return <div class="s1">{props.children}</div>
}

export function S2(props: Props) {
	return <div class="s2">{props.children}</div>
}

export function SpaceBetween(props: Props) {
	return <div class="space-between">{props.children}</div>
}

type HProps = {
	center?: boolean
	children?: Children
}

export function H1(props: HProps) {
	return <h1 class={cx('h1', props.center && 'centered')}>{props.children}</h1>
}

export function H2(props: HProps) {
	return <h2 class={cx('h2', props.center && 'centered')}>{props.children}</h2>
}

export function P(props: Props) {
	return <p class="p">{props.children}</p>
}

export function List(props: Props) {
	return <ul class="list">{props.children}</ul>
}

export function OrderedList(props: Props) {
	return <ol class="ordered-list">{props.children}</ol>
}

export function ListItem(props: Props) {
	return <li>{props.children}</li>
}
