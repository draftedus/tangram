import { Level } from './alert'
import { Children, css, cssClass, h, useCss } from './deps'
import { colors, variables } from './theme'

type CalloutProps = {
	children?: Children
	level: Level
	title?: string
}

let layoutWrapperClass = cssClass()
let layoutWrapperCss = css({
	[`.${layoutWrapperClass}`]: {
		display: 'grid',
		gridGap: '.5rem',
		padding: '1rem',
	},
})

let infoWrapperClass = cssClass()
let infoWrapperCss = css({
	[`.${infoWrapperClass}`]: {
		backgroundColor: colors.teal,
		borderLeft: `4px solid ${colors.teal}`,
	},
})

let warningWrapperClass = cssClass()
let warningWrapperCss = css({
	[`.${warningWrapperClass}`]: {
		backgroundColor: colors.yellow,
		borderLeft: `4px solid ${colors.yellow}`,
	},
})

let dangerWrapperClass = cssClass()
let dangerWrapperCss = css({
	[`.${dangerWrapperClass}`]: {
		backgroundColor: colors.red,
		borderLeft: `4px solid ${colors.red}`,
	},
})

let titleClass = cssClass()
let titleCss = css({
	[`.${titleClass}`]: {
		color: variables.colors.funText,
		fontWeight: 'bold',
	},
})

let innerClass = cssClass()
let innerCss = css({
	[`.${innerClass}`]: {
		color: variables.colors.funText,
		lineHeight: '1.5',
	},
})

export function Callout(props: CalloutProps) {
	let wrapperCss
	let wrapperClass
	switch (props.level) {
		case Level.Danger:
			wrapperCss = dangerWrapperCss
			wrapperClass = dangerWrapperClass
			break
		case Level.Info:
			wrapperCss = infoWrapperCss
			wrapperClass = infoWrapperClass
			break
		case Level.Warning:
			wrapperCss = warningWrapperCss
			wrapperClass = warningWrapperClass
			break
	}
	useCss(wrapperCss)
	useCss(layoutWrapperCss)
	useCss(titleCss)
	useCss(innerCss)

	return (
		<div class={wrapperClass}>
			<div class={layoutWrapperClass}>
				{props.title && <div class={titleClass}>{props.title}</div>}
				<div class={innerClass}>{props.children}</div>
			</div>
		</div>
	)
}
