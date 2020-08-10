import { h, ui } from 'deps'

export function Explanations() {
	let xAxisLabelFormatter = (i: number) => data.classes[i]
	let data = {
		chartData: [
			{ x: 0, y: 0.02517283 },
			{ x: 1, y: 0.9748272 },
		],
		className: 'Positive',
		classes: ['Negative', 'Positive'],
		probability: 0.9748272,
		shapChartData: [
			{
				baseline: 0.02783647,
				baselineLabel: '51%',
				label: 'Positive',
				output: 3.6564934,
				outputLabel: '97%',
				values: [
					{ feature: "thallium_stress_test = 'normal'", value: -0.39572704 },
					{ feature: "resting_blood_pressure = '160'", value: -0.14125186 },
					{ feature: "gender = 'male'", value: 0 },
					{
						feature: "fasting_blood_sugar_greater_than_120 = 'false'",
						value: 0,
					},
					{
						feature:
							"resting_ecg_result = 'probable or definite left ventricular hypertrophy'",
						value: 0,
					},
					{ feature: "exercise_induced_angina = 'yes'", value: 0 },
					{ feature: "fluoroscopy_vessels_colored = '3'", value: 0 },
					{ feature: "age = '67'", value: 0.13252445 },
					{ feature: "exercise_st_slope = 'flat'", value: 0.17108941 },
					{ feature: "exercise_st_depression = '1.5'", value: 0.22638911 },
					{ feature: "chest_pain = 'asymptomatic'", value: 0.7210883 },
					{ feature: "exercise_max_heart_rate = '108'", value: 1.1283911 },
					{ feature: "cholesterol = '286'", value: 1.7861533 },
				],
			},
		],
	}

	return (
		<ui.Window>
			<div class="explanations-wrapper">
				<div class="inner-wrapper">
					<ui.Card>
						<ui.NumberChart title="Predicted Class" value={data.className} />
					</ui.Card>
					<ui.Card>
						<ui.NumberChart
							title="Probability"
							value={ui.formatPercent(data.probability)}
						/>
					</ui.Card>
				</div>
				{data.classes.length > 2 && (
					<ui.BarChart
						data={[
							{
								color: ui.baseColors.blue,
								data: data.chartData,
								title: 'probabilities',
							},
						]}
						title="Predicted Probabilities"
						xAxisLabelFormatter={xAxisLabelFormatter}
					/>
				)}
				<ui.Card>
					<ui.ShapChart
						data={data.shapChartData}
						includeXAxisTitle={true}
						includeYAxisLabels={true}
						includeYAxisTitle={true}
						negativeColor="var(--red)"
						positiveColor="var(--green)"
					/>
				</ui.Card>
			</div>
		</ui.Window>
	)
}
