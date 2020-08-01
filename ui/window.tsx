import { Children, css, h, useCss } from './deps'
import { border, colors, variables } from './theme'

export enum WindowShade {
	Code,
	Default,
}

type WindowProps = {
	children?: Children
}

let wrapperCss = css({
	[`.window-wrapper`]: {
		backgroundColor: variables.colors.background,
		borderRadius: variables.border.radius,
		display: 'grid',
		grid: 'auto 1fr / minmax(0, 1fr)',
	},
})

let topbarCss = css({
	[`.window-topbar`]: {
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

let topbarButtonCss = css({
	[`.window-topbar-button`]: {
		borderRadius: '.375rem',
		boxSizing: 'border-box',
		height: '.75rem',
		width: '.75rem',
	},
})

let bodyCss = css({
	[`.window-body`]: {
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
		<div class="window-wrapper">
			<div class="window-topbar">
				<div
					class="window-topbar-button"
					style={{ backgroundColor: colors.red }}
				/>
				<div
					class="window-topbar-button"
					style={{ backgroundColor: colors.yellow }}
				/>
				<div
					class="window-topbar-button"
					style={{ backgroundColor: colors.green }}
				/>
			</div>
			<div class="window-body">{props.children}</div>
		</div>
	)
}
