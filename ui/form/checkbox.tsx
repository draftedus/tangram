import { css, h, useCss } from '../deps'
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

let checkboxFieldCss = css({
	[`.form-checkbox-field`]: {
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
	[`.form-checkbox-field:checked`]: {
		backgroundColor: variables.colors.accent,
		borderColor: variables.colors.accent,
	},
	[`.form-checkbox-field:checked:after`]: {
		color: variables.colors.funText,
		content: '"✔"',
	},
	[`.form-checkbox-field:hover`]: {
		borderColor: variables.colors.hover,
	},
	[`.form-checkbox-field:focus`]: {
		borderColor: variables.colors.accent,
	},
	[`.form-checkbox-field::-webkit-input-placeholder`]: {
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
				class="form-checkbox-field"
				name={props.name}
				placeholder={props.placeholder}
				readOnly={props.readOnly}
				type="checkbox"
				value={props.value ?? undefined}
			/>
		</Label>
	)
}
