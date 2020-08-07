import { h } from './deps'

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

export function Slider(props: SliderProps) {
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
		<div class="slider-wrapper">
			<input
				class="slider-range"
				max={props.max}
				min={props.min}
				onInput={onInput}
				type="range"
			/>
			<div class="slider-progress" style={progressStyle} />
			<div class="tooltip slider-tooltip" style={tooltipStyle}>
				{value}
			</div>
		</div>
	)
}
