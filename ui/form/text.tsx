import { css, cssClass, cx, h, useCss } from '../deps'
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
	type?: string
	value?: string | null
}

let textFieldClass = cssClass()
let textFieldCss = css({
	[`.${textFieldClass}`]: {
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

let fileClass = cssClass()
let fileCss = css({
	[`.${fileClass}`]: {
		backgroundColor: undefined,
		border: undefined,
		padding: undefined,
	},
})

export function TextField(props: TextFieldProps) {
	useCss(textFieldCss, fileCss)
	return (
		<Label>
			{props.label}
			<input
				autocomplete={props.autocomplete ?? 'off'}
				class={cx(textFieldClass, props.type === 'file' && fileClass)}
				name={props.name}
				placeholder={props.placeholder}
				readOnly={props.readOnly}
				spellcheck={false}
				type={props.type}
				value={props.value ?? undefined}
			/>
		</Label>
	)
}
