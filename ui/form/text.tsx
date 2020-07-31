import { css, cssClass, h, useCss } from '../deps'
import { border, variables } from '../theme'
import { Label } from './label'

type TextFieldProps = {
	autocomplete?: string
	disabled?: boolean
	label?: string
	name?: string
	onChange?: (newValue: string | null) => void
	placeholder?: string
	readOnly?: boolean
	value?: string | null
}

let textFieldClass = cssClass()
let textFieldCss = css({
	[`.${textFieldClass}`]: {
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
		padding: `calc(0.5rem - ${variables.border.width}) 1rem`,
		userSelect: 'text',
		width: '100%',
	},
	[`.${textFieldClass}:hover`]: {
		borderColor: variables.colors.hover,
	},
	[`.${textFieldClass}:focus`]: {
		borderColor: variables.colors.accent,
	},
	[`.${textFieldClass}::-webkit-input-placeholder`]: {
		WebkitTextFillColor: variables.colors.mutedText,
		color: variables.colors.mutedText,
	},
})

export function TextField(props: TextFieldProps) {
	useCss(textFieldCss)
	return (
		<Label>
			{props.label}
			<input
				autocomplete={props.autocomplete}
				class={textFieldClass}
				name={props.name}
				placeholder={props.placeholder}
				readOnly={props.readOnly}
				spellcheck={false}
				value={props.value ?? undefined}
			/>
		</Label>
	)
}
