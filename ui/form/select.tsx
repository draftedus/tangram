import { Children, css, h, useCss } from '../deps'
import { border, variables } from '../theme'
import { Label } from './label'

export type SelectProps = {
	children?: Children
	disabled?: boolean
	id?: string
	label?: string
	name?: string
	onChange?: (newValue: string | null) => void
	options?: string[]
	placeholder?: string
	required?: boolean
	value?: string | null
}

let selectCss = css({
	[`.form-select`]: {
		MozAppearance: 'none',
		WebkitTextFillColor: 'inherit',
		appearance: 'none',
		backgroundColor: variables.colors.surface,
		border,
		borderRadius: variables.border.radius,
		boxSizing: 'border-box',
		color: 'inherit',
		cursor: 'pointer',
		font: 'inherit',
		fontSize: '1rem',
		height: '2.5rem',
		outline: '1px none',
		padding: `calc(0.5rem - ${variables.border.width}) 1rem`,
		userSelect: 'text',
		width: '100%',
	},
	[`.form-select:hover`]: {
		borderColor: variables.colors.hover,
	},
	[`.form-select:focus`]: {
		borderColor: variables.colors.accent,
	},
	[`.form-select::-webkit-input-placeholder`]: {
		WebkitTextFillColor: variables.colors.mutedText,
		color: variables.colors.mutedText,
	},
})

export function SelectField(props: SelectProps) {
	useCss(selectCss)
	return (
		<Label>
			{props.label}
			<select
				class="form-select"
				disabled={props.disabled}
				id={props.id}
				name={props.name}
				onChange={event => {
					props.onChange?.(event.currentTarget.value)
				}}
				placeholder={props.placeholder}
				required={props.required}
				value={props.value ?? undefined}
			>
				{props.options
					? props.options.map(option => (
							<option key={option} selected={props.value == option}>
								{option}
							</option>
					  ))
					: props.children}
			</select>
		</Label>
	)
}
