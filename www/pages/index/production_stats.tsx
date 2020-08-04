import { css, h, ui, useCss } from 'deps'

let wrapperCss = css({
	[`.production-stats-wrapper`]: {
		display: 'grid',
		grid: 'auto auto / auto',
		gridGap: '1rem',
		gridTemplateAreas: `"alert" "comparison"`,
		padding: '1rem',
	},
})

export function ProductionStats() {
	useCss(wrapperCss)
	let xAxisLabelFormatter = (index: number): string => {
		let categories: string[] = [
			'asymptomatic',
			'atypical angina',
			'non-angina pain',
			'typical angina',
		]
		return categories[index]
	}
	let series = [
		{
			color: ui.colors.blue,
			data: [
				{ x: 0, y: 0.4752 },
				{ x: 1, y: 0.165 },
				{ x: 2, y: 0.2838 },
				{ x: 3, y: 0.07591 },
			],
			title: 'Training',
		},
		{
			color: ui.colors.green,
			data: [
				{ x: 0, y: 0 },
				{ x: 1, y: 0.1622 },
				{ x: 2, y: 0.2903 },
				{ x: 3, y: 0.07508 },
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
							xAxisLabelFormatter={xAxisLabelFormatter}
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
