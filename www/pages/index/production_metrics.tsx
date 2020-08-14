import { h, ui } from 'deps'

export function ProductionMetrics() {
	return (
		<ui.Window>
			<div class="production-metrics-wrapper">
				<ui.Card>
					<ui.LineChart
						data={accuracyData}
						title="Monthly Accuracy"
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
			{ label: accuracyLabels[0], x: 0, y: 0.8360655903816223 },
			{ label: accuracyLabels[1], x: 1, y: 0.8360655903816223 },
			{ label: accuracyLabels[2], x: 2, y: 0.8360655903816223 },
			{ label: accuracyLabels[3], x: 3, y: 0.8360655903816223 },
			{ label: accuracyLabels[4], x: 4, y: 0.8360655903816223 },
			{ label: accuracyLabels[5], x: 5, y: 0.8360655903816223 },
			{ label: accuracyLabels[6], x: 6, y: 0.8360655903816223 },
			{ label: accuracyLabels[7], x: 7, y: 0.8360655903816223 },
			{ label: accuracyLabels[8], x: 8, y: 0.8360655903816223 },
			{ label: accuracyLabels[9], x: 9, y: 0.8360655903816223 },
			{ label: accuracyLabels[10], x: 10, y: 0.8360655903816223 },
			{ label: accuracyLabels[11], x: 11, y: 0.8360655903816223 },
		],
		lineStyle: 2,
		pointStyle: 0,
		title: 'Training Accuracy',
	},
	{
		color: ui.colors.green,
		data: [
			{ label: accuracyLabels[0], x: 0, y: 0.827037 },
			{ label: accuracyLabels[1], x: 1, y: 0.83504676 },
			{ label: accuracyLabels[2], x: 2, y: 0.81508476 },
			{ label: accuracyLabels[3], x: 3, y: 0.8296226 },
			{ label: accuracyLabels[4], x: 4, y: 0.79173913 },
			{ label: accuracyLabels[5], x: 5, y: 0.77857144 },
			{ label: accuracyLabels[6], x: 6, y: null },
			{ label: accuracyLabels[7], x: 7, y: null },
			{ label: accuracyLabels[8], x: 8, y: null },
			{ label: accuracyLabels[9], x: 9, y: null },
			{ label: accuracyLabels[10], x: 10, y: null },
			{ label: accuracyLabels[11], x: 11, y: null },
		],
		title: 'Production Accuracy',
	},
]
