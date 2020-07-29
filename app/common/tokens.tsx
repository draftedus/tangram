import { h, ui } from 'deps'

export let trainingColor = ui.colors.blue
export let trainingTextColor = ui.variables.colors.funText

export let productionColor = ui.colors.green
export let productionTextColor = ui.variables.colors.funText

export let baselineColor = ui.colors.gray
export let baselineTextColor = ui.variables.colors.funText

export let selectedThresholdColor = ui.colors.blue
export let selectedThresholdTextColor = ui.variables.colors.funText

export function UnknownColumnToken() {
	return (
		<ui.Token color={ui.colors.gray} textColor={ui.variables.colors.funText}>
			Unknown
		</ui.Token>
	)
}

export function NumberColumnToken() {
	return (
		<ui.Token color={ui.colors.teal} textColor={ui.variables.colors.funText}>
			Number
		</ui.Token>
	)
}

export function EnumColumnToken() {
	return (
		<ui.Token color={ui.colors.purple} textColor={ui.variables.colors.funText}>
			Enum
		</ui.Token>
	)
}

export function TextColumnToken() {
	return (
		<ui.Token color={ui.colors.orange} textColor={ui.variables.colors.funText}>
			Text
		</ui.Token>
	)
}
