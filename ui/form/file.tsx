import { css, cssClass, h, useCss } from '../deps'
import { border, variables } from '../theme'
import { Label } from './label'

type FileFieldProps = {
	disabled?: boolean
	label?: string
	name?: string
}

let inputClass = cssClass()
let wrapperClass = cssClass()
let fileFieldCss = css({
	[`.${wrapperClass}`]: {
		backgroundColor: variables.colors.surface,
		border,
		borderRadius: variables.border.radius,
		boxSizing: 'border-box',
		color: 'inherit',
		cursor: 'pointer',
		font: 'inherit',
		fontSize: '1rem',
		height: '2.5rem',
		outline: 'none',
		padding: '0.5rem 1rem',
		position: 'relative',
		userSelect: 'text',
		width: '100%',
	},
	[`.${inputClass}`]: {
		MozAppearance: 'none',
		WebkitAppearance: 'none',
		WebkitTextFillColor: 'inherit',
		appearance: 'none',
	},
	[`.${inputClass}:hover`]: {
		borderColor: variables.colors.hover,
	},
	[`.${inputClass}:focus`]: {
		borderColor: variables.colors.accent,
	},
})

export function FileField(props: FileFieldProps) {
	useCss(fileFieldCss)
	return (
		<Label>
			{props.label}
			<div class={wrapperClass}>
				<input class={inputClass} name={props.name} type="file" />
			</div>
		</Label>
	)
}
