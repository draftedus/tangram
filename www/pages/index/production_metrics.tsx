import './production_metrics.css'
import { LineChart } from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

export function ProductionMetrics() {
	return (
		<ui.Window>
			<div class="production-metrics-wrapper">
				<ui.Card>
					<LineChart
						data={accuracyData}
						labels={accuracyLabels}
						title="Monthly Accuracy"
						xAxisGridLineInterval={{ k: 1, p: 0 }}
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
		color: ui.colors.blue,
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
		color: ui.colors.green,
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
