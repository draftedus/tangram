import { Children, css, cssClass, h, useCss } from './deps'
import { border, colors, variables } from './theme'

export enum WindowShade {
	Code,
	Default,
}

type WindowProps = {
	children?: Children
}

let wrapperClass = cssClass()
let wrapperCss = css({
	[`.${wrapperClass}`]: {
		backgroundColor: variables.colors.background,
		borderRadius: variables.border.radius,
		display: 'grid',
		grid: 'auto 1fr / minmax(0, 1fr)',
	},
})

let topbarClass = cssClass()
let topbarCss = css({
	[`.${topbarClass}`]: {
		backgroundColor: variables.colors.surface,
		borderLeft: border,
		borderRight: border,
		borderTop: border,
		borderTopLeftRadius: variables.border.radius,
		borderTopRightRadius: variables.border.radius,
		display: 'grid',
		grid: 'auto / auto auto auto',
		gridColumnGap: '.5rem',
		justifyContent: 'start',
		padding: '.75rem',
	},
})

let topbarButtonClass = cssClass()
let topbarButtonCss = css({
	[`.${topbarButtonClass}`]: {
		borderRadius: '.375rem',
		boxSizing: 'border-box',
		height: '.75rem',
		width: '.75rem',
	},
})

let bodyClass = cssClass()
let bodyCss = css({
	[`.${bodyClass}`]: {
		backgroundColor: variables.colors.surface,
		borderBottom: border,
		borderBottomLeftRadius: variables.border.radius,
		borderBottomRightRadius: variables.border.radius,
		borderLeft: border,
		borderRight: border,
	},
})

export function Window(props: WindowProps) {
	useCss(wrapperCss, topbarCss, topbarButtonCss, bodyCss)
	return (
		<div class={wrapperClass}>
			<div class={topbarClass}>
				<div
					class={topbarButtonClass}
					style={{ backgroundColor: colors.red }}
				/>
				<div
					class={topbarButtonClass}
					style={{ backgroundColor: colors.yellow }}
				/>
				<div
					class={topbarButtonClass}
					style={{ backgroundColor: colors.green }}
				/>
			</div>
			<div class={bodyClass}>{props.children}</div>
		</div>
	)
}
