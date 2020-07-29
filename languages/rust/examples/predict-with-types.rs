#![allow(dead_code)]

use tangram_rust as tangram;

#[derive(Debug, serde::Serialize)]
struct Input {
	age: f32,
	gender: Gender,
	chest_pain: ChestPain,
	resting_blood_pressure: f32,
	cholesterol: f32,
	fasting_blood_sugar_greater_than_120: FastingBloodSugarGreaterThan120,
	resting_ecg_result: RestingECGResult,
	exercise_max_heart_rate: f32,
	exercise_induced_angina: ExerciseInducedAngina,
	exercise_st_depression: f32,
	exercise_st_slope: ExerciseSTSlope,
	fluoroscopy_vessels_colored: FluoroscopyVesselsColored,
	thallium_stress_test: ThalliumStressTest,
}

#[derive(Debug, serde::Serialize)]
enum Gender {
	#[serde(rename = "male")]
	Male,
	#[serde(rename = "female")]
	Female,
}

#[derive(Debug, serde::Serialize)]
enum ChestPain {
	#[serde(rename = "asymptomatic")]
	Asymptomatic,
	#[serde(rename = "non-angina pain")]
	NonAnginaPain,
	#[serde(rename = "atypical angina")]
	AtypicalAngina,
	#[serde(rename = "typical angina")]
	TypicalAngina,
}

#[derive(Debug, serde::Serialize)]
enum FastingBloodSugarGreaterThan120 {
	False,
	True,
}

#[derive(Debug, serde::Serialize)]
enum RestingECGResult {
	#[serde(rename = "normal")]
	Normal,
	#[serde(rename = "probable or definite left ventricular hypertrophy")]
	LVH,
	#[serde(rename = "ST-T wave abnormality")]
	STTWaveAbnormality,
}

#[derive(Debug, serde::Serialize)]
enum ExerciseInducedAngina {
	#[serde(rename = "no")]
	No,
	#[serde(rename = "yes")]
	Yes,
}

#[derive(Debug, serde::Serialize)]
enum ExerciseSTSlope {
	#[serde(rename = "upsloping")]
	Upsloping,
	#[serde(rename = "flat")]
	Flat,
	#[serde(rename = "downsloping")]
	Downsloping,
}

#[derive(Debug, serde::Serialize)]
enum FluoroscopyVesselsColored {
	#[serde(rename = "0")]
	Zero,
	#[serde(rename = "1")]
	One,
	#[serde(rename = "2")]
	Two,
	#[serde(rename = "3")]
	Three,
}

#[derive(Debug, serde::Serialize)]
enum ThalliumStressTest {
	#[serde(rename = "normal")]
	Normal,
	#[serde(rename = "reversible defect")]
	ReversibleDefect,
	#[serde(rename = "fixed defect")]
	FixedDefect,
}

type Output = tangram::ClassificationOutput<Diagnosis>;

#[derive(serde::Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Diagnosis {
	Negative,
	Positive,
}

fn main() {
	// Load the model from the file.
	let model = tangram::Model::<Input, Output>::from_file("examples/heart-disease.tangram");

	// Create an example input matching the schema of the CSV file the model was trained on. Here the data is just hard-coded, but in your application you will probably get this from a database or user input.
	let input = Input {
		age: 63.0,
		gender: Gender::Male,
		chest_pain: ChestPain::TypicalAngina,
		resting_blood_pressure: 145.0,
		cholesterol: 233.0,
		fasting_blood_sugar_greater_than_120: FastingBloodSugarGreaterThan120::True,
		resting_ecg_result: RestingECGResult::LVH,
		exercise_max_heart_rate: 150.0,
		exercise_induced_angina: ExerciseInducedAngina::No,
		exercise_st_depression: 2.3,
		exercise_st_slope: ExerciseSTSlope::Downsloping,
		fluoroscopy_vessels_colored: FluoroscopyVesselsColored::One,
		thallium_stress_test: ThalliumStressTest::FixedDefect,
	};

	// Make the prediction!
	let output = model.predict(&[&input], None);

	// Print out the input and output.
	println!("{:?}", input);
	println!("{:?}", output);
}
