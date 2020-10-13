import './production_stats.css'
import { BarChart } from '@tangramhq/charts'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

export function ProductionStats() {
	let options: string[] = [
		'asymptomatic',
		'atypical angina',
		'non-angina pain',
		'typical angina',
	]
	let series = [
		{
			color: ui.colors.blue,
			data: [
				{ label: options[0], x: 0, y: 0.4752 },
				{ label: options[1], x: 1, y: 0.165 },
				{ label: options[2], x: 2, y: 0.2838 },
				{ label: options[3], x: 3, y: 0.07591 },
			],
			title: 'Training',
		},
		{
			color: ui.colors.green,
			data: [
				{ label: options[0], x: 0, y: 0 },
				{ label: options[1], x: 1, y: 0.1622 },
				{ label: options[2], x: 2, y: 0.2903 },
				{ label: options[3], x: 3, y: 0.07508 },
			],
			title: 'Production',
		},
	]
	return (
		<ui.Window>
			<div class="production-stats-wrapper">
				<div style={{ gridArea: 'alert' }}>
					<ui.Alert level={ui.Level.Danger}>{'High Invalid Count'}</ui.Alert>
				</div>
				<div style={{ gridArea: 'comparison' }}>
					<ui.Card>
						<BarChart
							data={series}
							title="Chest Pain"
							xAxisTitle="Chest Pain"
							yAxisTitle="Percent"
							yMax={1}
						/>
					</ui.Card>
				</div>
			</div>
		</ui.Window>
	)
}
