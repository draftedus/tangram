import './training_stats.css'
import { BoxChart } from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

export function TrainingStats() {
	let series = [
		{
			color: ui.colors.blue,
			data: [
				{
					label: 'Age',
					x: 0,
					y: {
						max: 77,
						min: 29,
						p25: 48,
						p50: 56,
						p75: 61,
					},
				},
			],
			title: 'quartiles',
		},
	]
	return (
		<ui.Window>
			<div class="training-stats-wrapper">
				<div style={{ gridArea: 'mean' }}>
					<ui.Card>
						<ui.NumberChart title="Mean" value="54.4389" />
					</ui.Card>
				</div>
				<div style={{ gridArea: 'std' }}>
					<ui.Card>
						<ui.NumberChart title="Standard Deviation" value="9.02374" />
					</ui.Card>
				</div>
				<div style={{ gridArea: 'age' }}>
					<ui.Card>
						<BoxChart
							data={series}
							shouldDrawXAxisLabels={false}
							title="Age"
							xAxisTitle="Age"
							yAxisTitle="Count"
						/>
					</ui.Card>
				</div>
			</div>
		</ui.Window>
	)
}
