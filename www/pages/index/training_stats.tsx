import { css, h, ui, useCss } from 'deps'

let wrapperCss = css({
	[`.training-stats-wrapper`]: {
		display: 'grid',
		grid: 'auto / auto auto',
		gridGap: '1rem',
		gridTemplateAreas: `"mean std" "age age"`,
		padding: '1rem',
	},
})
export function TrainingStats() {
	useCss(wrapperCss)
	let series = [
		{
			color: ui.colors.blue,
			data: [
				{
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
						<ui.BoxChart
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
