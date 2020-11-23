import { FieldLabel } from "./field_label"
import "./select_field.css"
import { h } from "preact"

export type SelectFieldProps = {
	disabled?: boolean
	id?: string
	label?: string
	name?: string
	options: SelectFieldOption[]
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
				placeholder={props.placeholder}
				required={props.required}
				value={props.value}
			>
				{props.options.map(({ text, value }) => (
					<option key={value} value={value}>
						{text}
					</option>
				))}
			</select>
		</FieldLabel>
	)
}

export function selectFieldSubmitOnChange(id: string) {
	let selectElement = document.getElementById(id)
	if (!(selectElement instanceof HTMLSelectElement)) throw Error()
	selectElement.addEventListener("change", event => {
		if (!(event.currentTarget instanceof HTMLSelectElement)) throw Error()
		let form = event.currentTarget.closest("form")
		if (!(form instanceof HTMLFormElement)) throw Error()
		form.submit()
	})
}
