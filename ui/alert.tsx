import { Children, css, h, useCss } from './deps'
import { colors, variables } from './theme'

export enum Level {
	Info,
	Warning,
	Danger,
}

export type AlertProps = {
	children?: Children
	level: Level
	title?: string
}

let alertWrapperCss = css({
	[`.alert-wrapper`]: {
		borderRadius: variables.border.radius,
		padding: '1rem',
	},
})

let infoCss = css({
	[`.alert-level-info`]: {
		backgroundColor: colors.teal,
		color: variables.colors.funText,
	},
})

let warningCss = css({
	[`.alert-level-warning`]: {
		backgroundColor: colors.yellow,
		color: variables.colors.funText,
	},
})

let dangerCss = css({
	[`.alert-level-danger`]: {
		backgroundColor: colors.red,
		color: variables.colors.funText,
	},
})

let titleCss = css({
	[`.alert-title`]: {
		color: variables.colors.funText,
		fontWeight: 'bold',
		marginBottom: '1rem',
	},
})

let textCss = css({
	[`.alert-text`]: {
		lineHeight: '1.5',
	},
})

export function Alert(props: AlertProps) {
	useCss(alertWrapperCss, titleCss, textCss)
	let levelClass
	let levelCss
	switch (props.level) {
		case Level.Info:
			levelCss = infoCss
			levelClass = 'alert-level-info'
			break
		case Level.Warning:
			levelCss = warningCss
			levelClass = 'alert-level-warning'
			break
		case Level.Danger:
			levelCss = dangerCss
			levelClass = 'alert-level-danger'
			break
	}
	useCss(levelCss)
	return (
		<div class={`alert-wrapper ${levelClass}`}>
			{props.title && <div class="alert-title">{props.title}</div>}
		</div>
	)
}
