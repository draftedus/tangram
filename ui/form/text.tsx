import { h } from '../deps'
import { Label } from './label'

type TextFieldProps = {
	autocomplete?: string
	disabled?: boolean
	label?: string
	name?: string
	onChange?: (newValue: string | null) => void
	placeholder?: string
	readOnly?: boolean
	required?: boolean
	value?: string | null
}

export function TextField(props: TextFieldProps) {
	return (
		<Label>
			{props.label}
			<input
				autocomplete={props.autocomplete}
				class="form-text-field"
				name={props.name}
				placeholder={props.placeholder}
				readOnly={props.readOnly}
				required={props.required}
				spellcheck={false}
				value={props.value ?? undefined}
			/>
		</Label>
	)
}
