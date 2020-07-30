import { css, cssClass, h, useCss } from '../deps'
import { border, variables } from '../theme'
import { Label } from './label'

type CheckboxFieldProps = {
	autocomplete?: string
	disabled?: boolean
	label?: string
	name?: string
	onChange?: (newValue: string | null) => void
	placeholder?: string
	readOnly?: boolean
	value?: string | null
}

let checkboxFieldClass = cssClass()
let checkboxFieldCss = css({
	[`.${checkboxFieldClass}`]: {
		MozAppearance: 'none',
		WebkitAppearance: 'none',
		alignItems: 'center',
		border,
		borderRadius: variables.border.radius,
		display: 'grid',
		height: '2.5rem',
		justifyItems: 'center',
		width: '2.5rem',
	},
	[`.${checkboxFieldClass}:checked`]: {
		backgroundColor: variables.colors.accent,
		borderColor: variables.colors.accent,
	},
	[`.${checkboxFieldClass}:checked:after`]: {
		color: variables.colors.funText,
		content: '"✔"',
	},
	[`.${checkboxFieldClass}:hover`]: {
		borderColor: variables.colors.hover,
	},
	[`.${checkboxFieldClass}:focus`]: {
		borderColor: variables.colors.accent,
	},
	[`.${checkboxFieldClass}::-webkit-input-placeholder`]: {
		WebkitTextFillColor: variables.colors.mutedText,
		color: variables.colors.mutedText,
	},
})

export function CheckboxField(props: CheckboxFieldProps) {
	useCss(checkboxFieldCss)
	return (
		<Label>
			{props.label}
			<input
				class={checkboxFieldClass}
				name={props.name}
				placeholder={props.placeholder}
				readOnly={props.readOnly}
				type="checkbox"
				value={props.value ?? undefined}
			/>
		</Label>
	)
}
