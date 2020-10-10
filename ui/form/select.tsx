import { Label } from './label'
import './select.css'
import { ComponentChildren, h } from 'preact'

export type SelectProps = {
	children?: ComponentChildren
	disabled?: boolean
	id?: string
	label?: string
	name?: string
	onChange?: (newValue: string) => void
	options?: SelectOption[]
	placeholder?: string
	required?: boolean
	value?: string
}

export type SelectOption = {
	text: string
	value: string
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
					? props.options.map(({ text, value }) => (
							<option key={value} value={value}>
								{text}
							</option>
					  ))
					: props.children}
			</select>
		</Label>
	)
}

export function selectSubmitOnChange(id: string) {
	let selectElement = document.getElementById(id)
	if (!(selectElement instanceof HTMLSelectElement)) throw Error()
	selectElement.addEventListener('change', event => {
		if (!(event.currentTarget instanceof HTMLSelectElement)) throw Error()
		let form = event.currentTarget.closest('form')
		if (!(form instanceof HTMLFormElement)) throw Error()
		form.submit()
	})
}
