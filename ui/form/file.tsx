import './file.css'
import { Label } from './label'
import { h } from 'preact'

type FileFieldProps = {
	disabled?: boolean
	label?: string
	name?: string
	required?: boolean
	value?: string
}

export function FileField(props: FileFieldProps) {
	return (
		<Label>
			{props.label}
			<div class="form-file-wrapper">
				{'Choose File'}
				<input
					class="form-file-input"
					name={props.name}
					required={props.required}
					type="file"
					value={props.value}
				/>
			</div>
		</Label>
	)
}

/**
 * When using a custom 'Choose File' prompt,
 * it is necessary to use JS to update it to
 * show the selected file name.
 */
export function bootFileFields() {
	let fileInputElements = document.querySelectorAll('input[type=file]')
	fileInputElements.forEach(fileInputElement => {
		if (!(fileInputElement instanceof HTMLInputElement)) throw Error()
		updateFileInputElement(fileInputElement)
		fileInputElement.addEventListener('change', () =>
			updateFileInputElement(fileInputElement),
		)
	})
}

function updateFileInputElement(fileInputElement: HTMLInputElement) {
	let file = fileInputElement.files?.item(0)
	if (file) {
		fileInputElement.parentElement?.firstChild?.replaceWith(
			document.createTextNode(file.name),
		)
	}
}
