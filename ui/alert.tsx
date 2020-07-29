import { Children, css, cssClass, cx, h, useCss } from './deps'
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

let wrapperClass = cssClass()
let wrapperCss = css({
	[`.${wrapperClass}`]: {
		borderRadius: variables.border.radius,
		padding: '1rem',
	},
})

let infoClass = cssClass()
let infoCss = css({
	[`.${infoClass}`]: {
		backgroundColor: colors.teal,
		color: variables.colors.funText,
	},
})

let warningClass = cssClass()
let warningCss = css({
	[`.${warningClass}`]: {
		backgroundColor: colors.yellow,
		color: variables.colors.funText,
	},
})

let dangerClass = cssClass()
let dangerCss = css({
	[`.${dangerClass}`]: {
		backgroundColor: colors.red,
		color: variables.colors.funText,
	},
})

let titleClass = cssClass()
let titleCss = css({
	[`.${titleClass}`]: {
		color: variables.colors.funText,
		fontWeight: 'bold',
		marginBottom: '1rem',
	},
})

let textClass = cssClass()
let textCss = css({
	[`.${textClass}`]: {
		lineHeight: '1.5',
	},
})

export function Alert(props: AlertProps) {
	useCss(wrapperCss, titleCss, textCss)
	let levelClass
	let levelCss
	switch (props.level) {
		case Level.Info:
			levelCss = infoCss
			levelClass = infoClass
			break
		case Level.Warning:
			levelCss = warningCss
			levelClass = warningClass
			break
		case Level.Danger:
			levelCss = dangerCss
			levelClass = dangerClass
			break
	}
	useCss(levelCss)
	return (
		<div class={cx(wrapperClass, levelClass)}>
			{props.title && <div class={titleClass}>{props.title}</div>}
			<div class={textClass}>{props.children}</div>
		</div>
	)
}
