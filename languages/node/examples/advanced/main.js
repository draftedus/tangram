let fs = require('fs')
let path = require('path')
let tangram = require('@tangramhq/tangram-node')

// If you are running the Tangram reporting and monitoring web app on-prem you can pass the URL to its API with the TANGRAM_URL environment variable.
let baseUrl = process.env.TANGRAM_URL || 'https://app.tangramhq.com.com'

// Get the path to the .tangram file.
let modelPath = path.join(__dirname, 'heart-disease.tangram')
// Load the model from the file and set the url where the tangram app is running.
let modelData = fs.readFileSync(modelPath)
let model = new tangram.Model(modelData, {
	tangramUrl: baseUrl,
})

// Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
let input = {
	age: 63,
	chest_pain: 'typical angina',
	cholesterol: 233,
	exercise_induced_angina: 'no',
	exercise_max_heart_rate: 150,
	exercise_st_depression: 2.3,
	exercise_st_slope: 'downsloping',
	fasting_blood_sugar_greater_than_120: 'true',
	fluoroscopy_vessels_colored: 0,
	gender: 'male',
	resting_blood_pressure: 145,
	resting_ecg_result: 'probable or definite left ventricular hypertrophy',
	thallium_stress_test: 'fixed defect',
}

// Make the prediction using a custom threshold chosen on the "Tuning" page of the Tangram reporting and monitoring web app.
options = { threshold: 0.5 }
let output = model.predictSync(input, options)

// Print out the input and output.
console.log('Input:', input)
console.log('Output:', output)

// Log the prediction. This will allow us to view production stats in the Tangram reporting and monitoring web app.
model.logPrediction({
	identifier: '6c955d4f-be61-4ca7-bba9-8fe32d03f801',
	input,
	options,
	output,
})

// Later on, if we get an official diagnosis for the patient, we can log the true value for the prior prediction. Make sure to match the `identifier` from the former prediction.
model.logTrueValue({
	identifier: '6c955d4f-be61-4ca7-bba9-8fe32d03f801',
	trueValue: 'Positive',
})
