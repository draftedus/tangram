import { Level } from './alert'
import { css, cssClass, cx, h, useCss } from './deps'
import { colors, variables } from './theme'

type AlertProps = {
	alert: string
	level: Level
}

let alertContainerClass = cssClass()
let alertContainerCss = css({
	[`.${alertContainerClass}`]: {
		cursor: 'pointer',
		position: 'relative',
	},
})

let alertMessageClass = cssClass()
let alertMessageCss = css({
	[`.${alertMessageClass}`]: {
		borderRadius: variables.border.radius,
		bottom: '1rem',
		color: variables.colors.funText,
		left: '1rem',
		padding: '.5rem',
		position: 'absolute',
		visibility: 'hidden',
	},
	[`.${alertMessageClass}:hover`]: {
		visibility: 'visible',
	},
})

let alertIconClass = cssClass()
let alertIconCss = css({
	[`.${alertIconClass}`]: {
		alignContent: 'center',
		borderRadius: '.5rem',
		color: variables.colors.funText,
		display: 'inline-grid',
		fontSize: '.8rem',
		height: '1rem',
		justifyContent: 'center',
		width: '1rem',
	},
	[`.${alertIconClass}:hover`]: {
		filter: 'brightness(90%)',
	},
})

let infoClass = cssClass()
let infoCss = css({
	[`.${infoClass}`]: {
		backgroundColor: colors.teal,
	},
})

let warningClass = cssClass()
let warningCss = css({
	[`.${warningClass}`]: {
		backgroundColor: colors.yellow,
	},
})

let dangerClass = cssClass()
let dangerCss = css({
	[`.${dangerClass}`]: {
		backgroundColor: colors.red,
	},
})

export function AlertIcon(props: AlertProps) {
	useCss(alertContainerCss, alertMessageCss, alertIconCss)
	let levelClass
	let levelCss
	switch (props.level) {
		case Level.Info:
			levelClass = infoClass
			levelCss = infoCss
			break
		case Level.Warning:
			levelClass = warningClass
			levelCss = warningCss
			break
		case Level.Danger:
			levelClass = dangerClass
			levelCss = dangerCss
			break
	}
	useCss(levelCss)

	let alertMessageClassCombined = cx(alertMessageClass, levelClass)
	let alertIconClassCombined = cx(alertIconClass, levelClass)

	return (
		<div class={alertContainerClass}>
			<div class={alertMessageClassCombined}>{props.alert}</div>
			<div class={alertIconClassCombined}>!</div>
		</div>
	)
}
