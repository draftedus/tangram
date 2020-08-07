import { h } from '../deps'
import { Label } from './label'

type CheckboxFieldProps = {
	autocomplete?: string
	disabled?: boolean
	label?: string
	name?: string
	onChange?: (newValue: string | null) => void
	placeholder?: string
	readOnly?: boolean
	value?: string | null
}

export function CheckboxField(props: CheckboxFieldProps) {
	return (
		<Label>
			{props.label}
			<input
				class="form-checkbox-field"
				name={props.name}
				placeholder={props.placeholder}
				readOnly={props.readOnly}
				type="checkbox"
				value={props.value ?? undefined}
			/>
		</Label>
	)
}
