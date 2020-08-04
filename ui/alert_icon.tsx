import { Level } from './alert'
import { css, cx, h, useCss } from './deps'
import { colors, variables } from './theme'

type AlertProps = {
	alert: string
	level: Level
}

let alertWrapperCss = css({
	[`.alert-icon-wrapper`]: {
		cursor: 'pointer',
		position: 'relative',
	},
})

let alertMessageCss = css({
	[`.alert-icon-message`]: {
		borderRadius: variables.border.radius,
		bottom: '1rem',
		color: variables.colors.funText,
		left: '1rem',
		padding: '.5rem',
		position: 'absolute',
		visibility: 'hidden',
	},
	[`.alert-icon-message:hover`]: {
		visibility: 'visible',
	},
})

let alertIconCss = css({
	[`.alert-icon`]: {
		alignContent: 'center',
		borderRadius: '.5rem',
		color: variables.colors.funText,
		display: 'inline-grid',
		fontSize: '.8rem',
		height: '1rem',
		justifyContent: 'center',
		width: '1rem',
	},
	[`.alert-icon:hover`]: {
		filter: 'brightness(90%)',
	},
})

let infoCss = css({
	[`.alert-icon-level-info`]: {
		backgroundColor: colors.teal,
	},
})

let warningCss = css({
	[`.alert-icon-level-warning`]: {
		backgroundColor: colors.yellow,
	},
})

let dangerCss = css({
	[`.alert-icon-level-danger`]: {
		backgroundColor: colors.red,
	},
})

export function AlertIcon(props: AlertProps) {
	useCss(alertWrapperCss, alertMessageCss, alertIconCss)
	let levelClass
	let levelCss
	switch (props.level) {
		case Level.Info:
			levelClass = 'alert-icon-level-info'
			levelCss = infoCss
			break
		case Level.Warning:
			levelClass = 'alert-icon-level-warning'
			levelCss = warningCss
			break
		case Level.Danger:
			levelClass = 'alert-icon-level-danger'
			levelCss = dangerCss
			break
	}
	useCss(levelCss)

	let alertMessageClassCombined = cx('alert-icon-message', levelClass)
	let alertIconClassCombined = cx('alert-icon', levelClass)

	return (
		<div class="alert-icon-container">
			<div class={alertMessageClassCombined}>{props.alert}</div>
			<div class={alertIconClassCombined}>{'!'}</div>
		</div>
	)
}
