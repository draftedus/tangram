import { css, h, useCss } from '../deps'
import { border, variables } from '../theme'
import { Label } from './label'

type FileFieldProps = {
	disabled?: boolean
	label?: string
	name?: string
}

let fileFieldCss = css({
	[`.form-file-field-wrapper`]: {
		backgroundColor: variables.colors.surface,
		border,
		borderRadius: variables.border.radius,
		boxSizing: 'border-box',
		cursor: 'pointer',
		fontSize: '1rem',
		height: '2.5rem',
		lineHeight: 1.5,
		outline: 'none',
		padding: `calc(0.5rem - ${variables.border.width}) 1rem`,
		position: 'relative',
		userSelect: 'text',
		width: '100%',
	},
	[`.form-file-field-wrapper:hover`]: {
		borderColor: variables.colors.hover,
	},
	[`.form-file-field-wrapper:focus`]: {
		borderColor: variables.colors.accent,
	},
	[`.form-file-input`]: {
		bottom: 0,
		left: 0,
		position: 'absolute',
		right: 0,
		top: 0,
		visibility: 'hidden',
		width: '100%',
	},
})

export function FileField(props: FileFieldProps) {
	useCss(fileFieldCss)
	return (
		<Label>
			{props.label}
			<div class="form-file-field-wrapper">
				Choose File
				<input class="form-file-input" name={props.name} type="file" />
			</div>
		</Label>
	)
}
