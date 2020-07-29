import modelData from '../heart-disease.tangram'
import * as tangram from '@tangramhq/tangram'
import React, { useCallback, useState } from 'react'

// Create the model with the data from the .tangram file
let model = new tangram.Model(modelData)

// Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
let input = {
	age: 63,
	chest_pain: 'typical angina',
	cholesterol: 233,
	exercise_induced_angina: 'no',
	exercise_max_heart_rate: 150,
	exercise_st_depression: 2.3,
	exercise_st_slope: 'downsloping',
	fasting_blood_sugar_greater_than_120: true,
	fluoroscopy_vessels_colored: 0,
	gender: 'male',
	resting_blood_pressure: 145,
	resting_ecg_result: 'probable or definite left ventricular hypertrophy',
	thallium_stress_test: 'fixed defect',
}

function App() {
	let [output, setOutput] = useState(null)
	let onClick = useCallback(() => {
		// Make the prediction!
		let output = model.predictSync(input)
		setOutput(output)
	}, [])
	return (
		<div>
			<h2>Input:</h2>
			<pre>{JSON.stringify(input, null, 2)}</pre>
			<button onClick={onClick}>Click to predict</button>
			{output && (
				<>
					<h2>Output:</h2>
					<pre>{JSON.stringify(output, null, 2)}</pre>
				</>
			)}
		</div>
	)
}

export default App
