import { ComponentChildren, h } from '../deps'

type Props = {
	children?: ComponentChildren
	for?: string
}

export function Label(props: Props) {
	return (
		<label class="form-label" htmlFor={props.for}>
			{props.children}
		</label>
	)
}
