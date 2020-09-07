use tangram_rust as tangram;

fn main() {
	// Load the model from the file.
	let model = tangram::Model::from_slice(include_bytes!("./heart-disease.tangram"));

	// Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
	let input = serde_json::json!({
		"age": 63,
		"gender": "male",
		"chest_pain": "typical angina",
		"resting_blood_pressure": 145,
		"cholesterol": 233,
		"fasting_blood_sugar_greater_than_120": "true",
		"resting_ecg_result": "probable or definite left ventricular hypertrophy",
		"exercise_max_heart_rate": 150,
		"exercise_induced_angina": "no",
		"exercise_st_depression": 2.3,
		"exercise_st_slope": "downsloping",
		"fluoroscopy_vessels_colored": 0,
		"thallium_stress_test": "fixed defect",
	});

	// Make the prediction!
	let output: Vec<tangram::Output> = model.predict(&[&input], None);

	// Print out the input and output.
	println!("{:?}", input);
	println!("{:?}", output);
}
