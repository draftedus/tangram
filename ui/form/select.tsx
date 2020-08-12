import { Children, h } from '../deps'
import { Label } from './label'

export type SelectProps = {
	children?: Children
	disabled?: boolean
	id?: string
	label?: string
	name?: string
	onChange?: (newValue: string) => void
	options?: string[]
	placeholder?: string
	required?: boolean
	value?: string
}

export function SelectField(props: SelectProps) {
	return (
		<Label>
			{props.label}
			<select
				class="form-select"
				disabled={props.disabled}
				id={props.id}
				name={props.name}
				onChange={event => {
					props.onChange?.(event.currentTarget.value)
				}}
				placeholder={props.placeholder}
				required={props.required}
				value={props.value}
			>
				{props.options
					? props.options.map(option => (
							<option key={option} selected={props.value == option}>
								{option}
							</option>
					  ))
					: props.children}
			</select>
		</Label>
	)
}
