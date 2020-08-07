import { h, ui } from 'deps'

export let trainingColor = 'var(--blue)'
export let productionColor = 'var(--green)'
export let baselineColor = 'var(--gray)'
export let selectedThresholdColor = 'var(--blue)'

export function UnknownColumnToken() {
	return <ui.Token color="var(--gray)">{'Unknown'}</ui.Token>
}

export function NumberColumnToken() {
	return <ui.Token color="var(--teal)">{'Number'}</ui.Token>
}

export function EnumColumnToken() {
	return <ui.Token color="var(--purple)">{'Enum'}</ui.Token>
}

export function TextColumnToken() {
	return <ui.Token color="var(--orange)">{'Text'}</ui.Token>
}
