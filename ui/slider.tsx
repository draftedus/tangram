import { css, cssClass, h, useCss } from './deps'
import { border, colors, variables } from './theme'

type SliderProps = {
	color: string
	max: number
	min: number
	onChange?: (value: number) => void
	showTicks?: boolean
	step: number
	value: number
	valueFormatter?: (value: number) => string
}

let wrapperClass = cssClass()
let wrapperCss = css({
	[`.${wrapperClass}`]: {
		height: '1rem',
		position: 'relative',
	},
	[`.${wrapperClass}:hover > .tooltip`]: { display: 'block' },
})

let rangeClass = cssClass()
let rangeCss = css({
	[`.${rangeClass}`]: {
		appearance: 'none',
		background: 'transparent',
		boxSizing: 'border-box',
		height: '1rem',
		left: '0',
		margin: '0',
		position: 'absolute',
		top: '0',
		width: '100%',
	},

	[`.${rangeClass}:focus`]: {
		outline: 'none',
	},
	[`.${rangeClass}::-webkit-slider-runnable-track`]: {
		backgroundColor: variables.colors.border,
		borderRadius: variables.border.radius,
		boxSizing: 'border-box',
		height: '4px',
	},
	[`.${rangeClass}::-webkit-slider-thumb`]: {
		WebkitAppearance: 'none',
		backgroundColor: colors.blue,
		borderRadius: '0.5rem',
		height: '1rem',
		marginTop: 'calc(-0.5rem + 2px)',
		position: 'relative',
		width: '1rem',
	},
	[`.${rangeClass}::-moz-range-track`]: {
		backgroundColor: variables.colors.border,
		borderRadius: variables.border.radius,
		boxSizing: 'border-box',
		height: '4px',
	},
	[`.${rangeClass}::-moz-range-progress`]: {
		backgroundColor: variables.colors.accent,
		borderRadius: variables.border.radius,
		boxSizing: 'border-box',
		height: '4px',
	},
	[`.${rangeClass}::-moz-range-thumb`]: {
		backgroundColor: variables.colors.accent,
		border: 'none',
		borderRadius: '0.5rem',
		display: 'none',
		height: '1rem',
		position: 'relative',
		width: '1rem',
	},
})

let tooltipClass = cssClass()
let tooltipCss = css({
	[`.${tooltipClass}`]: {
		backgroundColor: variables.colors.surface,
		border,
		borderRadius: variables.border.radius,
		display: 'none',
		padding: '0.5rem',
		position: 'absolute',
		textAlign: 'center',
		top: '-0.5rem',
		transform: 'translateX(-50%) translateY(-100%)',
		userSelect: 'none',
	},
})

let progressClass = cssClass()
let progressCss = css({
	[`.${progressClass}`]: {
		backgroundColor: variables.colors.accent,
		borderRadius: variables.border.radius,
		height: '4px',
		left: '0',
		position: 'absolute',
		right: '0',
		top: 'calc(0.5rem - 2px)',
	},
})

export function Slider(props: SliderProps) {
	useCss(wrapperCss, rangeCss, progressCss, tooltipCss)
	let value = props.valueFormatter
		? props.valueFormatter(props.value)
		: props.value
	let percent = ((props.value - props.min) / (props.max - props.min)) * 100
	let progressStyle = { width: `${percent}%` }
	let tooltipStyle = { marginLeft: `${percent}%` }
	let onInput = (event: Event) => {
		if (!(event.currentTarget instanceof HTMLInputElement)) throw Error()
		props.onChange?.(parseFloat(event.currentTarget.value))
	}
	return (
		<div class={wrapperClass}>
			<input
				class={rangeClass}
				max={props.max}
				min={props.min}
				onInput={onInput}
				type="range"
			/>
			<div class={progressClass} style={progressStyle} />
			<div class={'tooltip ' + tooltipClass} style={tooltipStyle}>
				{value}
			</div>
		</div>
	)
}
