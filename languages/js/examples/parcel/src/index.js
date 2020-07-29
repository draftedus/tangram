import * as tangram from '@tangramhq/tangram-js'
import { Buffer } from 'buffer'
import { readFileSync } from 'fs'

// Parcel searches for calls to readFileSync and inlines the referenced file in the bundle. You will need to import Buffer as shown above.
let modelData = readFileSync('src/heart-disease.tangram')
let model = new tangram.Model(modelData)

// Create an example input matching the schema from the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
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

// Make the prediction!
let output = model.predictSync(input)

// create some dom nodes to display the input and prediction
let inputJson = JSON.stringify(input, null, 2)
let inputNode = document.createElement('pre')
let inputTextNode = document.createTextNode('Input: ' + inputJson)
inputNode.appendChild(inputTextNode)
document.body.appendChild(inputNode)
let outputNode = document.createElement('pre')
let outputTextNode = document.createTextNode(
	'Output: ' + JSON.stringify(output, null, 2),
)
outputNode.appendChild(outputTextNode)
document.body.appendChild(outputNode)
