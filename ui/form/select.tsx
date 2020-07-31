import { Children, css, cssClass, h, useCss } from '../deps'
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
	value?: string | null
}

let selectClass = cssClass()
let selectCss = css({
	[`.${selectClass}`]: {
		MozAppearance: 'none',
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
		outline: '1px none',
		padding: '0.5rem 1rem',
		position: 'relative',
		userSelect: 'text',
		width: '100%',
	},
	[`.${selectClass}:hover`]: {
		borderColor: variables.colors.hover,
	},
	[`.${selectClass}:focus`]: {
		borderColor: variables.colors.accent,
	},
	[`.${selectClass}::-webkit-input-placeholder`]: {
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
				class={selectClass}
				disabled={props.disabled}
				id={props.id}
				name={props.name}
				onChange={event => {
					props.onChange?.(event.currentTarget.value)
				}}
				placeholder={props.placeholder}
				value={props.value ?? undefined}
			>
				{props.options
					? props.options.map(option => <option key={option}>{option}</option>)
					: props.children}
			</select>
		</Label>
	)
}
