import * as ui from '@tangramhq/ui'
import { Language } from 'layouts/language'
import { h } from 'preact'

export function Predict() {
	return <ui.CodeSelect languages={predictCodeForLanguage} />
}

let predictCodeForLanguage = {
	[Language.JavaScript]: `const fs = require('fs')
const path = require('path')
const tangram = require('@tangramhq/tangram')

// Get the path to the .tangram file.
const modelPath = path.join(__dirname, 'heart-disease.tangram')
// Load the model from the file.
const modelData = fs.readFileSync(modelPath)
const model = new tangram.Model(modelData)

// Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
const input = {
	age: 63,
	gender: 'male',
	chest_pain: 'typical angina',
	resting_blood_pressure: 145,
	cholesterol: 233,
	fasting_blood_sugar_greater_than_120: 'true',
	resting_ecg_result: 'probable or definite left ventricular hypertrophy',
	exercise_max_heart_rate: 150,
	exercise_induced_angina: 'no',
	exercise_st_depression: 2.3,
	exercise_st_slope: 'downsloping',
	fluoroscopy_vessels_colored: 0,
	thallium_stress_test: 'fixed defect',
}

// Make the prediction!
const output = model.predictSync(input)

// Print out the input and output.
console.log('Input:', input)
console.log('Output:', output)`,
	[Language.Python]: `import os
import tangram

# Get the path to the .tangram file.
model_path = os.path.join(os.path.dirname(__file__), 'heart-disease.tangram')
# Load the model from the file.
model = tangram.Model.from_file(model_path)

# Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
input = {
	'age': 63,
	'gender': 'male',
	'chest_pain': 'typical angina',
	'resting_blood_pressure': 145,
	'cholesterol': 233,
	'fasting_blood_sugar_greater_than_120': 'true',
	'resting_ecg_result': 'probable or definite left ventricular hypertrophy',
	'exercise_max_heart_rate': 150,
	'exercise_induced_angina': 'no',
	'exercise_st_depression': 2.3,
	'exercise_st_slope': 'downsloping',
	'fluoroscopy_vessels_colored': 0,
	'thallium_stress_test': 'fixed defect',
}

# Make the prediction!
output = model.predict(input)

# Print out the input and output.
print('Input:', input)
print('Output:', output)`,
	[Language.Ruby]: `require 'tangram'

# Get the path to the .tangram file.
model_path = File.join(File.dirname(__FILE__), 'heart-disease.tangram')
# Load the model from the file.
model = Tangram::Model.from_file(model_path)

# Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
input = {
	age: 63,
	gender: 'male',
	chest_pain: 'typical angina',
	resting_blood_pressure: 145,
	cholesterol: 233,
	fasting_blood_sugar_greater_than_120: 'true',
	resting_ecg_result: 'probable or definite left ventricular hypertrophy',
	exercise_max_heart_rate: 150,
	exercise_induced_angina: 'no',
	exercise_st_depression: 2.3,
	exercise_st_slope: 'downsloping',
	fluoroscopy_vessels_colored: 0,
	thallium_stress_test: 'fixed defect',
}

# Make the prediction!
output = model.predict(input)

# Print out the input and output.
puts('Input:', input)
puts('Output:', output)`,
	[Language.Go]: `import "github.com/tangram-hq/tangram/languages/go"

// Load the model from the file.
model, err := tangram.LoadModelFromFile("./heart-disease.tangram", nil)
if err != nil {
	log.Fatal(err)
}
// destroy the model when it is no longer needed to free up memory.
defer model.Destroy()

// Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
input := tangram.Input{
	"age":                                  63,
	"gender":                               "male",
	"chest_pain":                           "typical angina",
	"resting_blood_pressure":               145,
	"cholesterol":                          233,
	"fasting_blood_sugar_greater_than_120": "true",
	"resting_ecg_result":                   "probable or definite left ventricular hypertrophy",
	"exercise_max_heart_rate":              150,
	"exercise_induced_angina":              "no",
	"exercise_st_depression":               2.3,
	"exercise_st_slope":                    "downsloping",
	"fluoroscopy_vessels_colored":          0,
	"thallium_stress_test":                 "fixed defect",
}

// Make the prediction!
output := model.PredictOne(input, nil)

// Print out the input and output.
fmt.Println("Input:", input)
fmt.Println("Output:", output.ClassName)`,
}
