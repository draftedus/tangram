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
	let workclass_options = vec![
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
	let education_options = vec![
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
	let marital_status_options = vec![
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
	let occupation_options = vec![
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
	let relationship_options = vec![
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
	let race_options = vec![
		"White",
		"Black",
		"Asian-Pac-Islander",
		"Amer-Indian-Eskimo",
		"Other",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let sex_options = vec!["Male", "Female"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let native_country_options = vec![
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
	let income_options = vec!["<=50K", ">50K"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let options = tangram_dataframe::FromCsvOptions {
		column_types: Some(btreemap! {
		  "age".into() => DataFrameColumnType::Number ,
			"workclass".into() => DataFrameColumnType::Enum {options: workclass_options },
			"fnlwgt".into() =>
		   DataFrameColumnType::Number,
			"education".into() => DataFrameColumnType::Enum {options: education_options },
			"education_num".into() =>
		   DataFrameColumnType::Number,
			"marital_status".into() => DataFrameColumnType::Enum {options: marital_status_options },
			"occupation".into() => DataFrameColumnType::Enum {options: occupation_options },
			"relationship".into() => DataFrameColumnType::Enum {options: relationship_options },
			"race".into() => DataFrameColumnType::Enum {options: race_options },
			"sex".into() => DataFrameColumnType::Enum {options: sex_options },
			"capital_gain".into() =>
		   DataFrameColumnType::Number,
			"capital_loss".into() =>
		   DataFrameColumnType::Number,
			"hours_per_week".into() =>
		   DataFrameColumnType::Number,
			"native_country".into() => DataFrameColumnType::Enum {options: native_country_options },
			"income".into() => DataFrameColumnType::Enum {options: income_options },
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
	let model = tangram_tree::BinaryClassifier::train(
		features_train.view(),
		labels_train.view(),
		&train_options,
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
