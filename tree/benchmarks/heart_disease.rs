use itertools::izip;
use maplit::btreemap;
use ndarray::prelude::*;
use serde_json::json;
use std::path::Path;
use tangram_dataframe::prelude::*;
use tangram_metrics::Metric;

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/heart_disease_train.csv");
	let csv_file_path_test = Path::new("data/heart_disease_test.csv");
	let _n_rows_train = 242;
	let n_rows_test = 61;
	let target_column_index = 13;
	let gender_options = vec!["male", "female"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let chest_pain_options = vec![
		"typical angina",
		"asymptomatic",
		"non-angina pain",
		"atypical angina",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let fasting_blood_sugar_greater_than_120_options = vec!["True", "False"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let resting_ecg_result_options = vec![
		"probable or definite left ventricular hypertrophy",
		"normal",
		"ST-T wave abnormality",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let exercise_induced_angina_options =
		vec!["no", "yes"].iter().map(ToString::to_string).collect();
	let exercise_st_slope_options = vec!["downsloping", "flat", "upsloping"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let thallium_stress_test_options = vec!["fixed defect", "normal", "reversible defect"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let diagnosis_options = vec!["Negative", "Positive"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let options = tangram_dataframe::FromCsvOptions {
		column_types: Some(btreemap! {
		  "age".into() => DataFrameColumnType::Number,
		  "gender".into() => DataFrameColumnType::Enum {options: gender_options},
		  "chest_pain".into() => DataFrameColumnType::Enum {options: chest_pain_options},
		  "resting_blood_pressure".into() => DataFrameColumnType::Number ,
		  "cholesterol".into() => DataFrameColumnType::Number,
		  "fasting_blood_sugar_greater_than_120".into() => DataFrameColumnType::Enum {options: fasting_blood_sugar_greater_than_120_options},
		  "resting_ecg_result".into() => DataFrameColumnType::Enum {options: resting_ecg_result_options},
		  "exercise_max_heart_rate".into() => DataFrameColumnType::Number,
		  "exercise_induced_angina".into() => DataFrameColumnType::Enum {options: exercise_induced_angina_options},
		  "exercise_st_depression".into() => DataFrameColumnType::Number,
		  "exercise_st_slope".into() => DataFrameColumnType::Enum {options: exercise_st_slope_options},
		  "fluoroscopy_vessels_colored".into() => DataFrameColumnType::Number,
		  "thallium_stress_test".into() => DataFrameColumnType::Enum {options: thallium_stress_test_options},
		  "diagnosis".into() => DataFrameColumnType::Enum {options: diagnosis_options},
		}),
		..Default::default()
	};
	let mut features_train =
		DataFrame::from_path(csv_file_path_train, options.clone(), |_| {}).unwrap();
	let labels_train = features_train.columns_mut().remove(target_column_index);
	let mut features_test =
		DataFrame::from_path(csv_file_path_test, options.clone(), |_| {}).unwrap();
	let labels_test = features_test.columns_mut().remove(target_column_index);
	let labels_train = labels_train.as_enum().unwrap();
	let labels_test = labels_test.as_enum().unwrap();

	// Train the model.
	let model = tangram_tree::BinaryClassifier::train(
		features_train.view(),
		labels_train.view(),
		tangram_tree::TrainOptions {
			max_leaf_nodes: 255,
			..Default::default()
		},
		&mut |_| {},
	);

	// Make predictions on the test data.
	let features_test = features_test.to_rows();
	let mut probabilities = Array::zeros(n_rows_test);
	model.predict(features_test.view(), probabilities.view_mut());

	// Compute metrics.
	let input = izip!(probabilities.iter(), labels_test.iter())
		.map(|(probability, label)| (*probability, label.unwrap()))
		.collect();
	let auc_roc = tangram_metrics::AUCROC::compute(input);
	let output = json!({ "auc_roc": auc_roc });
	println!("{}", output);
}
