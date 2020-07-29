import { variables } from '../theme'

export type ConfusionMatrixConfig = {
	borderRadius: string
	falseBackgroundColor: string
	falseForegroundColor: string
	trueBackgroundColor: string
	trueForegroundColor: string
}

export let config: ConfusionMatrixConfig = {
	borderRadius: variables.border.radius,
	falseBackgroundColor: variables.colors.surface,
	falseForegroundColor: variables.colors.text,
	trueBackgroundColor: variables.colors.header,
	trueForegroundColor: variables.colors.text,
}
