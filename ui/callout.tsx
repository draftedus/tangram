import { Level } from './alert'
import { Children, css, h, useCss } from './deps'
import { colors, variables } from './theme'

type CalloutProps = {
	children?: Children
	level: Level
	title?: string
}

let layoutWrapperCss = css({
	[`.callout-wrapper`]: {
		display: 'grid',
		gridGap: '.5rem',
		padding: '1rem',
	},
})

let infoWrapperCss = css({
	[`.callout-level-info-wrapper`]: {
		backgroundColor: colors.teal,
		borderLeft: `4px solid ${colors.teal}`,
	},
})

let warningWrapperCss = css({
	[`.callout-level-warning-wrapper`]: {
		backgroundColor: colors.yellow,
		borderLeft: `4px solid ${colors.yellow}`,
	},
})

let dangerWrapperCss = css({
	[`.callout-level-danger-wrapper`]: {
		backgroundColor: colors.red,
		borderLeft: `4px solid ${colors.red}`,
	},
})

let titleCss = css({
	[`.callout-title`]: {
		color: variables.colors.funText,
		fontWeight: 'bold',
	},
})

let innerCss = css({
	[`.callout-inner`]: {
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
			wrapperClass = 'callout-level-danger-wrapper'
			break
		case Level.Info:
			wrapperCss = infoWrapperCss
			wrapperClass = 'callout-level-info-wrapper'
			break
		case Level.Warning:
			wrapperCss = warningWrapperCss
			wrapperClass = 'callout-level-warning-wrapper'
			break
	}
	useCss(wrapperCss, layoutWrapperCss, titleCss, innerCss)

	return (
		<div class={wrapperClass}>
			<div class="callout-wrapper">
				{props.title && <div class="callout-title">{props.title}</div>}
				<div class="callout-inner">{props.children}</div>
			</div>
		</div>
	)
}
