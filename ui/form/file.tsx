import { css, cssClass, h, useCss } from '../deps'
import { border, variables } from '../theme'
import { Label } from './label'

type FileFieldProps = {
	disabled?: boolean
	label?: string
	name?: string
}

let wrapperClass = cssClass()
let fileClass = cssClass()
let fileFieldCss = css({
	[`.${wrapperClass}`]: {
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
	[`.${fileClass}`]: {
		bottom: 0,
		left: 0,
		position: 'absolute',
		right: 0,
		top: 0,
		visibility: 'hidden',
		width: '100%',
	},
	[`.${wrapperClass}:hover`]: {
		borderColor: variables.colors.hover,
	},
	[`.${wrapperClass}:focus`]: {
		borderColor: variables.colors.accent,
	},
})

export function FileField(props: FileFieldProps) {
	useCss(fileFieldCss)
	return (
		<Label>
			{props.label}
			<div class={wrapperClass}>
				Choose File
				<input class={fileClass} name={props.name} type="file" />
			</div>
		</Label>
	)
}

export function hydrateFileFields() {
	fileClass
}
