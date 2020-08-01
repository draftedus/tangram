import { css, h, ui, useCss } from 'deps'

let wrapperCss = css({
	[`.production-metrics-wrapper`]: {
		padding: '1rem',
	},
	[ui.desktop]: {
		[`.production-metrics-wrapper`]: {
			grid: 'auto / auto',
			gridTemplateAreas: `"accuracy" "monthly"`,
		},
	},
})

export function ProductionMetrics() {
	useCss(wrapperCss)
	return (
		<ui.Window>
			<div class="production-metrics-wrapper">
				<ui.Card>
					<ui.LineChart
						data={accuracyData}
						title="Monthly Accuracy"
						xAxisLabelFormatter={i => accuracyLabels[i]}
						yMax={1}
						yMin={0}
					/>
				</ui.Card>
			</div>
		</ui.Window>
	)
}

let accuracyLabels = [
	'Jan 2020',
	'Feb 2020',
	'Mar 2020',
	'Apr 2020',
	'May 2020',
	'Jun 2020',
	'Jul 2020',
	'Aug 2020',
	'Sep 2020',
	'Oct 2020',
	'Nov 2020',
	'Dec 2020',
]

let accuracyData = [
	{
		color: '#0A84FF',
		data: [
			{ x: 0, y: 0.8360655903816223 },
			{ x: 1, y: 0.8360655903816223 },
			{ x: 2, y: 0.8360655903816223 },
			{ x: 3, y: 0.8360655903816223 },
			{ x: 4, y: 0.8360655903816223 },
			{ x: 5, y: 0.8360655903816223 },
			{ x: 6, y: 0.8360655903816223 },
			{ x: 7, y: 0.8360655903816223 },
			{ x: 8, y: 0.8360655903816223 },
			{ x: 9, y: 0.8360655903816223 },
			{ x: 10, y: 0.8360655903816223 },
			{ x: 11, y: 0.8360655903816223 },
		],
		lineStyle: 2,
		pointStyle: 0,
		title: 'Training Accuracy',
	},
	{
		color: '#30D158',
		data: [
			{ x: 0, y: 0.827037 },
			{ x: 1, y: 0.83504676 },
			{ x: 2, y: 0.81508476 },
			{ x: 3, y: 0.8296226 },
			{ x: 4, y: 0.79173913 },
			{ x: 5, y: 0.77857144 },
			{ x: 6, y: null },
			{ x: 7, y: null },
			{ x: 8, y: null },
			{ x: 9, y: null },
			{ x: 10, y: null },
			{ x: 11, y: null },
		],
		title: 'Production Accuracy',
	},
]
