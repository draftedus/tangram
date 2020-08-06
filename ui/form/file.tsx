import { css, h, useCss } from '../deps'
import { border, variables } from '../theme'
import { Label } from './label'

type FileFieldProps = {
	disabled?: boolean
	label?: string
	name?: string
	required?: boolean
}

let fileFieldCss = css({
	[`.form-file-wrapper`]: {
		backgroundColor: variables.colors.surface,
		border,
		borderRadius: variables.border.radius,
		boxSizing: 'border-box',
		fontSize: '1rem',
		height: '2.5rem',
		lineHeight: 1.5,
		outline: 'none',
		padding: `calc(0.5rem - ${variables.border.width}) 1rem`,
		position: 'relative',
		userSelect: 'text',
		width: '100%',
	},
	'.form-file-wrapper:hover': {
		borderColor: variables.colors.hover,
	},
	[`.form-file-wrapper:focus-within`]: {
		borderColor: variables.colors.accent,
	},
	[`.form-file-input`]: {
		bottom: 0,
		left: 0,
		opacity: 0,
		position: 'absolute',
		right: 0,
		top: 0,
		width: '100%',
	},
})

export function FileField(props: FileFieldProps) {
	useCss(fileFieldCss)
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
				/>
			</div>
		</Label>
	)
}

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
