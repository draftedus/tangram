import './field_label.css'
import { ComponentChildren, h } from 'preact'

type Props = {
	children?: ComponentChildren
	for?: string
}

export function FieldLabel(props: Props) {
	return (
		<label class="field-label" htmlFor={props.for}>
			{props.children}
		</label>
	)
}
