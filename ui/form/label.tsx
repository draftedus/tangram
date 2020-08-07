import { Children, h } from '../deps'

type Props = {
	children?: Children
	for?: string
}

export function Label(props: Props) {
	return (
		<label class="form-label" htmlFor={props.for}>
			{props.children}
		</label>
	)
}
