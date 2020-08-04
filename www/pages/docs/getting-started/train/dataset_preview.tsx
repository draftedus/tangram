import { h, ui } from 'deps'

export function DatasetPreview() {
	return (
		<ui.Table>
			<ui.TableHeader>
				<ui.TableRow>
					<ui.TableHeaderCell>{'age'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'gender'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'chest_pain'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'resting_blood_pressure'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'cholesterol'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'fasting_blood_sugar'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'resting_ecg_result'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'exercise_max_heart_rate'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'exercise_induced_angina'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'exercise_st_depression'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'exercise_st_slope'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>
						{'fluoroscopy_vessels_colored'}
					</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'thal'}</ui.TableHeaderCell>
					<ui.TableHeaderCell>{'diagnosis'}</ui.TableHeaderCell>
				</ui.TableRow>
			</ui.TableHeader>
			<ui.TableBody>
				{data.map((entry, i) => (
					<ui.TableRow key={i}>
						<ui.TableCell>{entry.age}</ui.TableCell>
						<ui.TableCell>{entry.gender}</ui.TableCell>
						<ui.TableCell>{entry.chestPain}</ui.TableCell>
						<ui.TableCell>{entry.restingBloodPressure}</ui.TableCell>
						<ui.TableCell>{entry.cholesterol}</ui.TableCell>
						<ui.TableCell>{entry.fastingBloodSugarGreaterThan120}</ui.TableCell>
						<ui.TableCell>{entry.restingEcgResult}</ui.TableCell>
						<ui.TableCell>{entry.exerciseMaxHeartRate}</ui.TableCell>
						<ui.TableCell>{entry.exerciseInducedAngina}</ui.TableCell>
						<ui.TableCell>{entry.exerciseStDepression}</ui.TableCell>
						<ui.TableCell>{entry.exerciseStSlope}</ui.TableCell>
						<ui.TableCell>{entry.fluoroscopyVesselsColored}</ui.TableCell>
						<ui.TableCell>{entry.thalliumStressTest}</ui.TableCell>
						<ui.TableCell>{entry.diagnosis}</ui.TableCell>
					</ui.TableRow>
				))}
			</ui.TableBody>
		</ui.Table>
	)
}

let data = [
	{
		age: '63',
		chestPain: 'typical angina',
		cholesterol: '233',
		diagnosis: 'Negative',
		exerciseInducedAngina: 'no',
		exerciseMaxHeartRate: '150',
		exerciseStDepression: '2.3',
		exerciseStSlope: 'downsloping',
		fastingBloodSugarGreaterThan120: 'true',
		fluoroscopyVesselsColored: '0.0',
		gender: 'male',
		restingBloodPressure: '145',
		restingEcgResult: 'probable or definite left ventricular hypertrophy',
		thalliumStressTest: 'fixed defect',
	},
	{
		age: '67',
		chestPain: 'asymptomatic',
		cholesterol: '286',
		diagnosis: 'Positive',
		exerciseInducedAngina: 'yes',
		exerciseMaxHeartRate: '108',
		exerciseStDepression: '1.5',
		exerciseStSlope: 'flat',
		fastingBloodSugarGreaterThan120: 'false',
		fluoroscopyVesselsColored: '3.0',
		gender: 'male',
		restingBloodPressure: '160',
		restingEcgResult: 'probable or definite left ventricular hypertrophy',
		thalliumStressTest: 'normal',
	},
	{
		age: '67',
		chestPain: 'asymptomatic',
		cholesterol: '229',
		diagnosis: 'Positive',
		exerciseInducedAngina: 'yes',
		exerciseMaxHeartRate: '129',
		exerciseStDepression: '2.6',
		exerciseStSlope: 'flat',
		fastingBloodSugarGreaterThan120: 'false',
		fluoroscopyVesselsColored: '2.0',
		gender: 'male',
		restingBloodPressure: '120',
		restingEcgResult: 'probable or definite left ventricular hypertrophy',
		thalliumStressTest: 'reversible defect',
	},
]
