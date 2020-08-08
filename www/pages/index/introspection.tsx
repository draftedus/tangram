import { h, ui } from 'deps'

export function Introspection() {
	let series = [
		{
			color: '#0a84ff',
			data: [
				{ x: 0, y: 0.7830284833908081 },
				{ x: 1, y: 0.5727340579032898 },
				{ x: 2, y: 0.5031881928443909 },
				{ x: 3, y: 0.42829298973083496 },
				{ x: 4, y: 0.28097838163375854 },
				{ x: 5, y: 0.27078115940093994 },
				{ x: 6, y: 0.24317367374897003 },
				{ x: 7, y: 0.23082280158996582 },
				{ x: 8, y: 0.23057544231414795 },
				{ x: 9, y: 0.14213471114635468 },
				{ x: 10, y: 0.13826103508472443 },
				{ x: 11, y: 0.1192748099565506 },
				{ x: 12, y: 0.05492711067199707 },
				{ x: 13, y: 0.0488085001707077 },
				{ x: 14, y: 0.035625942051410675 },
				{ x: 15, y: 0.03285684809088707 },
				{ x: 16, y: 0.03015032596886158 },
				{ x: 17, y: 0.011666063219308853 },
				{ x: 18, y: 0 },
				{ x: 19, y: 0 },
				{ x: 20, y: 0 },
				{ x: 21, y: 0 },
				{ x: 22, y: 0 },
				{ x: 23, y: 0 },
				{ x: 24, y: -0.011313040740787983 },
				{ x: 25, y: -0.02816474623978138 },
				{ x: 26, y: -0.10736455768346786 },
				{ x: 27, y: -0.10939698666334152 },
				{ x: 28, y: -0.1902274638414383 },
				{ x: 29, y: -0.20927897095680237 },
				{ x: 30, y: -0.2639586329460144 },
				{ x: 31, y: -0.3578639626502991 },
				{ x: 32, y: -0.4067492187023163 },
				{ x: 33, y: -0.4638928174972534 },
				{ x: 34, y: -0.6301491260528564 },
				{ x: 35, y: -0.7452244758605957 },
			],
			title: 'Feature Weight',
		},
	]
	let features = [
		"chest_pain == 'asymptomatic'",
		"thallium_stress_test == 'reversible defect'",
		'exercise_st_depression',
		"gender == 'male'",
		"fluoroscopy_vessels_colored == '1'",
		"fluoroscopy_vessels_colored == '2'",
		"fluoroscopy_vessels_colored == '3'",
		"exercise_induced_angina == 'yes'",
		"exercise_st_slope == 'flat'",
		'age',
		'resting_blood_pressure',
		"resting_ecg_result == 'probable or definite left ventricular hypertrophy'",
		"exercise_st_slope == 'downsloping'",
		"thallium_stress_test == 'fixed defect'",
		'cholesterol',
		"fasting_blood_sugar_greater_than_120 == 'false'",
		'thallium_stress_test OOV',
		"resting_ecg_result == 'ST-T wave abnormality'",
		'gender OOV',
		'chest_pain OOV',
		'fasting_blood_sugar_greater_than_120 OOV',
		'resting_ecg_result OOV',
		'exercise_induced_angina OOV',
		'exercise_st_slope OOV',
		"fasting_blood_sugar_greater_than_120 == 'true'",
		'fluoroscopy_vessels_colored OOV',
		"chest_pain == 'atypical angina'",
		"resting_ecg_result == 'normal'",
		"chest_pain == 'typical angina'",
		"exercise_induced_angina == 'no'",
		"exercise_st_slope == 'upsloping'",
		'exercise_max_heart_rate',
		"gender == 'female'",
		"chest_pain == 'non-angina pain'",
		"thallium_stress_test == 'normal'",
		"fluoroscopy_vessels_colored == '0'",
	]
	return (
		<ui.Window>
			<div class="introspection-wrapper">
				<ui.Card>
					<ui.BarChart
						data={series}
						shouldDrawXAxisLabels={false}
						title="Feature Weights"
						xAxisLabelFormatter={i => features[i]}
						xAxisTitle="Name"
						yAxisTitle="Weight"
					/>
				</ui.Card>
			</div>
		</ui.Window>
	)
}
