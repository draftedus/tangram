import os
import tangram

# If you are running the Tangram reporting and monitoring web app on your own server you can pass the URL to it with the TANGRAM_URL environment variable.
tangram_url = os.getenv('TANGRAM_URL', default='https://app.tangramhq.com.com')

# Get the path to the `.tangram` file.
model_path = os.path.join(os.path.dirname(__file__), 'heart_disease.tangram')
# Load the model from the file and set the url where the tangram is running.
model = tangram.Model.from_file(model_path, tangram_url=tangram_url)

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

# Make the prediction using a custom threshold chosen on the "Tuning" page of the Tangram reporting and monitoring web app.
options = { 'threshold': 0.25 }
output = model.predict(input, options)

# Print out the input and output.
print('Input:', input)
print('Output:', output)

# Log the prediction. This will allow us to view production stats in the Tangram reporting and monitoring web app.
model.log_prediction(
	identifier='John Doe',
	options=options,
	input=input,
	output=output,
)

# Later on, if we get an official diagnosis for the patient, we can log the true value for the prior prediction. Make sure to match the `identifier` from the former prediction.
model.log_true_value(
	identifier='John Doe',
	true_value='Positive',
)
