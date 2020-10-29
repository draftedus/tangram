use itertools::izip;
use maplit::btreemap;
use ndarray::prelude::*;
use serde_json::json;
use std::path::Path;
use tangram_dataframe::prelude::*;
use tangram_metrics::Metric;

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/census_train.csv");
	let csv_file_path_test = Path::new("data/census_test.csv");
	let _n_rows_train = 26049;
	let n_rows_test = 6512;
	let target_column_index = 14;
	let workclass_options = [
		"State-gov",
		"Self-emp-not-inc",
		"Private",
		"Federal-gov",
		"Local-gov",
		"?",
		"Self-emp-inc",
		"Without-pay",
		"Never-worked",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let education_options = [
		"Bachelors",
		"HS-grad",
		"11th",
		"Masters",
		"9th",
		"Some-college",
		"Assoc-acdm",
		"Assoc-voc",
		"7th-8th",
		"Doctorate",
		"Prof-school",
		"5th-6th",
		"10th",
		"1st-4th",
		"Preschool",
		"12th",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let marital_status_options = [
		"Never-married",
		"Married-civ-spouse",
		"Divorced",
		"Married-spouse-absent",
		"Separated",
		"Married-AF-spouse",
		"Widowed",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let occupation_options = [
		"Adm-clerical",
		"Exec-managerial",
		"Handlers-cleaners",
		"Prof-specialty",
		"Other-service",
		"Sales",
		"Craft-repair",
		"Transport-moving",
		"Farming-fishing",
		"Machine-op-inspct",
		"Tech-support",
		"?",
		"Protective-serv",
		"Armed-Forces",
		"Priv-house-serv",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let relationship_options = [
		"Not-in-family",
		"Husband",
		"Wife",
		"Own-child",
		"Unmarried",
		"Other-relative",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let race_options = [
		"White",
		"Black",
		"Asian-Pac-Islander",
		"Amer-Indian-Eskimo",
		"Other",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let sex_options = ["Male", "Female"].iter().map(ToString::to_string).collect();
	let native_country_options = [
		"United-States",
		"Cuba",
		"Jamaica",
		"India",
		"?",
		"Mexico",
		"South",
		"Puerto-Rico",
		"Honduras",
		"England",
		"Canada",
		"Germany",
		"Iran",
		"Philippines",
		"Italy",
		"Poland",
		"Columbia",
		"Cambodia",
		"Thailand",
		"Ecuador",
		"Laos",
		"Taiwan",
		"Haiti",
		"Portugal",
		"Dominican-Republic",
		"El-Salvador",
		"France",
		"Guatemala",
		"China",
		"Japan",
		"Yugoslavia",
		"Peru",
		"Outlying-US(Guam-USVI-etc)",
		"Scotland",
		"Trinadad&Tobago",
		"Greece",
		"Nicaragua",
		"Vietnam",
		"Hong",
		"Ireland",
		"Hungary",
		"Holand-Netherlands",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let income_options = ["<=50K", ">50K"].iter().map(ToString::to_string).collect();
	let options = tangram_dataframe::FromCsvOptions {
		column_types: Some(btreemap! {
		  "age".to_owned() => DataFrameColumnType::Number ,
			"workclass".to_owned() => DataFrameColumnType::Enum {options: workclass_options },
			"fnlwgt".to_owned() => DataFrameColumnType::Number,
			"education".to_owned() => DataFrameColumnType::Enum {options: education_options },
			"education_num".to_owned() => DataFrameColumnType::Number,
			"marital_status".to_owned() => DataFrameColumnType::Enum {options: marital_status_options },
			"occupation".to_owned() => DataFrameColumnType::Enum {options: occupation_options },
			"relationship".to_owned() => DataFrameColumnType::Enum {options: relationship_options },
			"race".to_owned() => DataFrameColumnType::Enum {options: race_options },
			"sex".to_owned() => DataFrameColumnType::Enum {options: sex_options },
			"capital_gain".to_owned() => DataFrameColumnType::Number,
			"capital_loss".to_owned() => DataFrameColumnType::Number,
			"hours_per_week".to_owned() => DataFrameColumnType::Number,
			"native_country".to_owned() => DataFrameColumnType::Enum {options: native_country_options },
			"income".to_owned() => DataFrameColumnType::Enum {options: income_options },
		}),
		..Default::default()
	};
	let mut features_train =
		DataFrame::from_path(csv_file_path_train, options.clone(), |_| {}).unwrap();
	let labels_train = features_train.columns_mut().remove(target_column_index);
	let labels_train = labels_train.as_enum().unwrap();
	let mut features_test =
		DataFrame::from_path(csv_file_path_test, options.clone(), |_| {}).unwrap();
	let labels_test = features_test.columns_mut().remove(target_column_index);
	let labels_test = labels_test.as_enum().unwrap();

	// Train the model.
	let train_options = tangram_tree::TrainOptions {
		learning_rate: 0.1,
		max_leaf_nodes: 255,
		max_rounds: 100,
		..Default::default()
	};
	let train_output = tangram_tree::BinaryClassifier::train(
		features_train.view(),
		labels_train.view(),
		&train_options,
		&mut |_| {},
	);

	// Make predictions on the test data.
	let features_test = features_test.to_rows();
	let mut probabilities = Array::zeros(n_rows_test);
	train_output
		.model
		.predict(features_test.view(), probabilities.view_mut());

	// Compute metrics.
	let input = izip!(probabilities.iter(), labels_test.iter())
		.map(|(probability, label)| (*probability, label.unwrap()))
		.collect();
	let auc_roc = tangram_metrics::AUCROC::compute(input);
	let output = json!({ "auc_roc": auc_roc });
	println!("{}", output);
}
