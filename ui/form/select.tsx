import { Label } from './label'
import { ComponentChildren, h } from 'preact'

export type SelectProps = {
	children?: ComponentChildren
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
							<option key={option} value={option}>
								{option}
							</option>
					  ))
					: props.children}
			</select>
		</Label>
	)
}
