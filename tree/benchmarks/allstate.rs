use maplit::btreemap;
use ndarray::prelude::*;
use serde_json::json;
use std::path::Path;
use tangram_dataframe::prelude::*;
use tangram_metrics::StreamingMetric;

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/allstate_train.csv");
	let csv_file_path_test = Path::new("data/allstate_test.csv");
	let target_column_index = 34;
	let blind_make_options = vec![
		"AU", "BF", "AR", "AJ", "BO", "BW", "BH", "AQ", "L", "BP", "AN", "K", "AO", "AH", "D", "X",
		"Y", "W", "BU", "Q", "R", "AL", "BV", "M", "I", "BG", "BT", "E", "S", "AY", "P", "N", "O",
		"AI", "Z", "BZ", "BY", "BM", "AX", "J", "BN", "BS", "AZ", "BB", "AV", "BD", "AF", "G",
		"AC", "AW", "?", "BR", "BA", "V", "AD", "AE", "B", "U", "AP", "AM", "BC", "CB", "AT", "BL",
		"F", "AG", "A", "AS", "BQ", "AK", "CA", "BK", "H", "C", "AB",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let blind_model_options = vec!["AU.54", "BF.36", "AR.41"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let blind_submodel_options = vec!["AU.54.3", "BF.36.3"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let cat1_options = vec!["G", "B", "D", "I", "F", "A", "E", "C", "H", "J", "?"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let cat2_options = vec!["B", "C", "?", "A"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let cat3_options = vec!["A", "F", "B", "C", "E", "D", "?"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let cat4_options = vec!["A", "?", "C", "B"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let cat5_options = vec!["C", "A", "?", "B"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let cat6_options = vec!["D", "C", "E", "B", "F", "?"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let cat7_options = vec!["A", "C", "?", "D", "B"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let cat8_options = vec!["A", "B", "C", "?"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let cat9_options = vec!["B", "A"].iter().map(ToString::to_string).collect();
	let cat10_options = vec!["A", "B", "C", "?"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let cat11_options = vec!["B", "E", "C", "A", "D", "F", "?"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let cat12_options = vec!["C", "B", "E", "D", "F", "A"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let ordcat_options = vec!["4", "2", "5", "3", "7", "6", "1", "?"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let nvcat_options = vec![
		"L", "N", "M", "J", "B", "E", "O", "F", "K", "D", "A", "H", "C", "G", "I",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let options = tangram_dataframe::FromCsvOptions {
		column_types: Some(btreemap! {
		"row_id".into()=> DataFrameColumnType::Number,
		"household_id".into()=> DataFrameColumnType::Number,
		"vehicle".into()=> DataFrameColumnType::Number,
		"calendar_year".into()=> DataFrameColumnType::Number,
		"model_year".into()=> DataFrameColumnType::Number,
		"blind_make".into()=> DataFrameColumnType::Enum { options: blind_make_options },
		"blind_model".into()=> DataFrameColumnType::Enum { options: blind_model_options },
		"blind_submodel".into()=> DataFrameColumnType::Enum { options: blind_submodel_options },
		"cat1".into()=> DataFrameColumnType::Enum { options: cat1_options },
		"cat2".into()=>  DataFrameColumnType::Enum { options: cat2_options },
		"cat3".into()=>  DataFrameColumnType::Enum { options: cat3_options },
		"cat4".into()=>  DataFrameColumnType::Enum { options: cat4_options },
		"cat5".into()=>  DataFrameColumnType::Enum { options: cat5_options },
		"cat6".into()=>  DataFrameColumnType::Enum { options: cat6_options },
		"cat7".into()=>  DataFrameColumnType::Enum { options: cat7_options },
		"cat8".into()=>  DataFrameColumnType::Enum { options: cat8_options },
		"cat9".into()=>  DataFrameColumnType::Enum { options: cat9_options },
		"cat10".into()=>  DataFrameColumnType::Enum { options: cat10_options },
		"cat11".into()=>  DataFrameColumnType::Enum { options: cat11_options },
		"cat12".into()=>  DataFrameColumnType::Enum { options: cat12_options },
		"ordcat".into()=> DataFrameColumnType::Enum { options: ordcat_options },
		"var1".into()=> DataFrameColumnType::Number,
		"var2".into()=> DataFrameColumnType::Number,
		"var3".into()=> DataFrameColumnType::Number,
		"var4".into()=> DataFrameColumnType::Number,
		"var5".into()=> DataFrameColumnType::Number,
		"var6".into()=> DataFrameColumnType::Number,
		"var7".into()=> DataFrameColumnType::Number,
		"var8".into()=> DataFrameColumnType::Number,
		"nvcat".into()=> DataFrameColumnType::Enum { options: nvcat_options },
		"nvvar2".into()=> DataFrameColumnType::Number,
		"nvvar3".into()=> DataFrameColumnType::Number,
		"nvvar4".into()=> DataFrameColumnType::Number ,
		"claim_amount".into()=> DataFrameColumnType::Number,
		}),
		..Default::default()
	};
	let mut features_train =
		DataFrame::from_path(csv_file_path_train, options.clone(), |_| {}).unwrap();
	let labels_train = features_train.columns_mut().remove(target_column_index);
	let labels_train = labels_train.as_number().unwrap();
	let mut features_test = DataFrame::from_path(csv_file_path_test, options, |_| {}).unwrap();
	let labels_test = features_test.columns_mut().remove(target_column_index);
	let labels_test = labels_test.as_number().unwrap();

	// Train the model.
	let start = std::time::Instant::now();
	let train_options = tangram_tree::TrainOptions {
		learning_rate: 0.1,
		max_leaf_nodes: 255,
		max_rounds: 100,
		..Default::default()
	};
	let model = tangram_tree::Regressor::train(
		features_train.view(),
		labels_train.view(),
		&train_options,
		&mut |_| {},
	);
	let duration = start.elapsed().as_secs_f32();

	// Make predictions on the test data.
	let features = features_test.to_rows();
	let mut predictions = Array::zeros(labels_test.len());
	model.predict(features.view(), predictions.view_mut());

	// Compute metrics.
	let mut metrics = tangram_metrics::RegressionMetrics::new();
	metrics.update(tangram_metrics::RegressionMetricsInput {
		predictions: predictions.as_slice().unwrap(),
		labels: labels_test.view().as_slice(),
	});
	let metrics = metrics.finalize();
	let output = json!({"mse": metrics.mse, "duration": duration});
	println!("{}", output);
}
