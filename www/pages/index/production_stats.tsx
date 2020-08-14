import { h, ui } from 'deps'

export function ProductionStats() {
	let categories: string[] = [
		'asymptomatic',
		'atypical angina',
		'non-angina pain',
		'typical angina',
	]
	let series = [
		{
			color: ui.colors.blue,
			data: [
				{ label: categories[0], x: 0, y: 0.4752 },
				{ label: categories[1], x: 1, y: 0.165 },
				{ label: categories[2], x: 2, y: 0.2838 },
				{ label: categories[3], x: 3, y: 0.07591 },
			],
			title: 'Training',
		},
		{
			color: ui.colors.green,
			data: [
				{ label: categories[0], x: 0, y: 0 },
				{ label: categories[1], x: 1, y: 0.1622 },
				{ label: categories[2], x: 2, y: 0.2903 },
				{ label: categories[3], x: 3, y: 0.07508 },
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
						<ui.BarChart
							data={series}
							title="Chest Pain"
							xAxisTitle="Chest Pain"
							yAxisLabelFormatter={value => ui.formatPercent(value, 2)}
							yAxisTitle="Percent"
							yMax={1}
						/>
					</ui.Card>
				</div>
			</div>
		</ui.Window>
	)
}
