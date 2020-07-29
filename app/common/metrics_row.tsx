import { Children, css, cssClass, h, ui, useCss } from 'deps'

type MetricsRowProps = {
	children: Children
}

let metricsRowClass = cssClass()
let metricsRowCss = css({
	[`.${metricsRowClass}`]: {
		display: 'grid',
		gridGap: '1rem',
	},
	[ui.desktop]: {
		[`.${metricsRowClass}`]: {
			grid: 'auto / 1fr',
		},
	},
	[ui.mobile]: {
		[`.${metricsRowClass}`]: {
			grid: 'auto / 1fr',
		},
	},
})

export function MetricsRow(props: MetricsRowProps) {
	useCss(metricsRowCss)
	return <div class={metricsRowClass}>{props.children}</div>
}
