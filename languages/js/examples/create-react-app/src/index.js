import React, { useState, useCallback } from 'react'
import ReactDOM from 'react-dom'
import * as tangram from '@tangramhq/tangram'
import inlineModel from '@tangramhq/inline-model.macro'

// Using the `inlineModel` macro, your model's data is inlined into your javascript bundle.
const modelData = inlineModel('./heart-disease.tangram')
// Create the model from the model data.
const model = new tangram.Model(modelData)

// Create an example input matching the schema from the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
const input = {
	age: 63,
	gender: 'male',
	chest_pain: 'typical angina',
	resting_blood_pressure: 145,
	cholesterol: 233,
	fasting_blood_sugar_greater_than_120: true,
	resting_ecg_result: 'probable or definite left ventricular hypertrophy',
	exercise_max_heart_rate: 150,
	exercise_induced_angina: 'no',
	exercise_st_depression: 2.3,
	exercise_st_slope: 'downsloping',
	fluoroscopy_vessels_colored: 0,
	thallium_stress_test: 'fixed defect',
}

// This is a React component that makes a prediction when the button is clicked.
const App = () => {
	const [output, setOutput] = useState(null)
	const onClick = useCallback(() => {
		// Make the prediction!
		const output = model.predictSync(input)
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

ReactDOM.render(<App />, document.getElementById('root'))
