import { Children, css, h, ui, useCss } from 'deps'

type MetricsRowProps = {
	children: Children
}

let metricsRowCss = css({
	[`.metrics-row`]: {
		display: 'grid',
		gridGap: '1rem',
	},
	[ui.desktop]: {
		[`.metrics-row`]: {
			gridAutoFlow: 'column',
		},
	},
	[ui.mobile]: {
		[`.metrics-row`]: {
			gridAutoFlow: 'row',
		},
	},
})

export function MetricsRow(props: MetricsRowProps) {
	useCss(metricsRowCss)
	return <div class="metrics-row">{props.children}</div>
}
