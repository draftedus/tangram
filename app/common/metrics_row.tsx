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
			gridAutoFlow: 'column',
		},
	},
	[ui.mobile]: {
		[`.${metricsRowClass}`]: {
			gridAutoFlow: 'row',
		},
	},
})

export function MetricsRow(props: MetricsRowProps) {
	useCss(metricsRowCss)
	return <div class={metricsRowClass}>{props.children}</div>
}
