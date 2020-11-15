import "./introspection.css"
import { BarChart } from "@tangramhq/charts"
import * as ui from "@tangramhq/ui"
import { h } from "preact"

export function Introspection() {
	return (
		<ui.Window>
			<div class="introspection-wrapper">
				<ui.Card>
					<BarChart
						data={data}
						shouldDrawXAxisLabels={false}
						title="Feature Weights"
						xAxisTitle="Name"
						yAxisTitle="Weight"
					/>
				</ui.Card>
			</div>
		</ui.Window>
	)
}

let data = [
	{
		color: ui.colors.blue,
		data: [
			{ label: "chest_pain == 'asymptomatic'", x: 0, y: 0.7830284833908081 },
			{
				label: "thallium_stress_test == 'reversible defect'",
				x: 1,
				y: 0.5727340579032898,
			},
			{ label: "exercise_st_depression", x: 2, y: 0.5031881928443909 },
			{ label: "gender == 'male'", x: 3, y: 0.42829298973083496 },
			{
				label: "fluoroscopy_vessels_colored == '1'",
				x: 4,
				y: 0.28097838163375854,
			},
			{
				label: "fluoroscopy_vessels_colored == '2'",
				x: 5,
				y: 0.27078115940093994,
			},
			{
				label: "fluoroscopy_vessels_colored == '3'",
				x: 6,
				y: 0.24317367374897003,
			},
			{
				label: "exercise_induced_angina == 'yes'",
				x: 7,
				y: 0.23082280158996582,
			},
			{ label: "exercise_st_slope == 'flat'", x: 8, y: 0.23057544231414795 },
			{ label: "age", x: 9, y: 0.14213471114635468 },
			{ label: "resting_blood_pressure", x: 10, y: 0.13826103508472443 },
			{
				label:
					"resting_ecg_result == 'probable or definite left ventricular hypertrophy'",
				x: 11,
				y: 0.1192748099565506,
			},
			{
				label: "exercise_st_slope == 'downsloping'",
				x: 12,
				y: 0.05492711067199707,
			},
			{
				label: "thallium_stress_test == 'fixed defect'",
				x: 13,
				y: 0.0488085001707077,
			},
			{ label: "cholesterol", x: 14, y: 0.035625942051410675 },
			{
				label: "fasting_blood_sugar_greater_than_120 == 'false'",
				x: 15,
				y: 0.03285684809088707,
			},
			{ label: "thallium_stress_test OOV", x: 16, y: 0.03015032596886158 },
			{
				label: "resting_ecg_result == 'ST-T wave abnormality'",
				x: 17,
				y: 0.011666063219308853,
			},
			{ label: "gender OOV", x: 18, y: 0 },
			{ label: "chest_pain OOV", x: 19, y: 0 },
			{ label: "fasting_blood_sugar_greater_than_120 OOV", x: 20, y: 0 },
			{ label: "resting_ecg_result OOV", x: 21, y: 0 },
			{ label: "exercise_induced_angina OOV", x: 22, y: 0 },
			{ label: "exercise_st_slope OOV", x: 23, y: 0 },
			{
				label: "fasting_blood_sugar_greater_than_120 == 'true'",
				x: 24,
				y: -0.011313040740787983,
			},
			{
				label: "fluoroscopy_vessels_colored OOV",
				x: 25,
				y: -0.02816474623978138,
			},
			{
				label: "chest_pain == 'atypical angina'",
				x: 26,
				y: -0.10736455768346786,
			},
			{
				label: "resting_ecg_result == 'normal'",
				x: 27,
				y: -0.10939698666334152,
			},
			{
				label: "chest_pain == 'typical angina'",
				x: 28,
				y: -0.1902274638414383,
			},
			{
				label: "exercise_induced_angina == 'no'",
				x: 29,
				y: -0.20927897095680237,
			},
			{
				label: "exercise_st_slope == 'upsloping'",
				x: 30,
				y: -0.2639586329460144,
			},
			{ label: "exercise_max_heart_rate", x: 31, y: -0.3578639626502991 },
			{ label: "gender == 'female'", x: 32, y: -0.4067492187023163 },
			{
				label: "chest_pain == 'non-angina pain'",
				x: 33,
				y: -0.4638928174972534,
			},
			{
				label: "thallium_stress_test == 'normal'",
				x: 34,
				y: -0.6301491260528564,
			},
			{
				label: "fluoroscopy_vessels_colored == '0'",
				x: 35,
				y: -0.7452244758605957,
			},
		],
		title: "Feature Weight",
	},
]
