import { FieldLabel } from './field_label'
import './select_field.css'
import { ComponentChildren, h } from 'preact'

export type SelectFieldProps = {
	children?: ComponentChildren
	disabled?: boolean
	id?: string
	label?: string
	name?: string
	onChange?: (newValue: string) => void
	options?: SelectFieldOption[]
	placeholder?: string
	required?: boolean
	value?: string
}

export type SelectFieldOption = {
	text: string
	value: string
}

export function SelectField(props: SelectFieldProps) {
	return (
		<FieldLabel>
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
					? props.options.map(({ text, value }) => (
							<option key={value} value={value}>
								{text}
							</option>
					  ))
					: props.children}
			</select>
		</FieldLabel>
	)
}

export function selectFieldSubmitOnChange(id: string) {
	let selectElement = document.getElementById(id)
	if (!(selectElement instanceof HTMLSelectElement)) throw Error()
	selectElement.addEventListener('change', event => {
		if (!(event.currentTarget instanceof HTMLSelectElement)) throw Error()
		let form = event.currentTarget.closest('form')
		if (!(form instanceof HTMLFormElement)) throw Error()
		form.submit()
	})
}
