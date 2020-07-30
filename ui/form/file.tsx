import { css, cssClass, h, useCss } from '../deps'
import { border, variables } from '../theme'
import { Label } from './label'

type FileFieldProps = {
	autocomplete?: string
	disabled?: boolean
	label?: string
	name?: string
	onChange?: (newValue: string | null) => void
	placeholder?: string
	readOnly?: boolean
	value?: string | null
}

let fileFieldClass = cssClass()
let fileFieldCss = css({
	[`.${fileFieldClass}`]: {
		MozAppearance: 'none',
		WebkitAppearance: 'none',
		WebkitTextFillColor: 'inherit',
		appearance: 'none',
		backgroundColor: variables.colors.surface,
		border,
		borderRadius: variables.border.radius,
		boxSizing: 'border-box',
		color: 'inherit',
		font: 'inherit',
		fontSize: '1rem',
		height: '2.5rem',
		outline: 'none',
		padding: '0.5rem 1rem',
		position: 'relative',
		userSelect: 'text',
		width: '100%',
	},
	[`.${fileFieldClass}:hover`]: {
		borderColor: variables.colors.hover,
	},
	[`.${fileFieldClass}:focus`]: {
		borderColor: variables.colors.accent,
	},
	[`.${fileFieldClass}::-webkit-input-placeholder`]: {
		WebkitTextFillColor: variables.colors.mutedText,
		color: variables.colors.mutedText,
	},
	[`.${fileFieldClass}::-webkit-file-upload-button`]: {
		visibility: 'hidden',
	},
})

export function FileField(props: FileFieldProps) {
	useCss(fileFieldCss)
	return (
		<Label>
			{props.label}
			<input
				autocomplete={props.autocomplete ?? 'off'}
				class={fileFieldClass}
				name={props.name}
				placeholder={props.placeholder}
				readOnly={props.readOnly}
				spellcheck={false}
				type="file"
				value={props.value ?? undefined}
			/>
		</Label>
	)
}
