use itertools::izip;
use maplit::btreemap;
use ndarray::prelude::*;
use std::path::Path;
use tangram_dataframe::*;
use tangram_metrics::StreamingMetric;

fn main() {
	// load the data
	// let csv_file_path = Path::new("data/higgs.csv");
	// let nrows_train = 10_500_000;
	// let nrows_test = 500_000;
	let csv_file_path = Path::new("data/higgs-small.csv");
	let nrows_train = 450_000;
	let nrows_test = 50_000;
	let target_column_index = 0;
	let options = FromCsvOptions {
		column_types: Some(btreemap! {
			"signal".to_owned() => ColumnType::Enum { options: vec!["false".into(), "true".into()] },
			"lepton_pt".to_owned() => ColumnType::Number,
			"lepton_eta".to_owned() => ColumnType::Number,
			"lepton_phi".to_owned() => ColumnType::Number,
			"missing_energy_magnitude".to_owned() => ColumnType::Number,
			"missing_energy_phi".to_owned() => ColumnType::Number,
			"jet_1_pt".to_owned() => ColumnType::Number,
			"jet_1_eta".to_owned() => ColumnType::Number,
			"jet_1_phi".to_owned() => ColumnType::Number,
			"jet_1_b_tag".to_owned() => ColumnType::Number,
			"jet_2_pt".to_owned() => ColumnType::Number,
			"jet_2_eta".to_owned() => ColumnType::Number,
			"jet_2_phi".to_owned() => ColumnType::Number,
			"jet_2_b_tag".to_owned() => ColumnType::Number,
			"jet_3_pt".to_owned() => ColumnType::Number,
			"jet_3_eta".to_owned() => ColumnType::Number,
			"jet_3_phi".to_owned() => ColumnType::Number,
			"jet_3_b_tag".to_owned() => ColumnType::Number,
			"jet_4_pt".to_owned() => ColumnType::Number,
			"jet_4_eta".to_owned() => ColumnType::Number,
			"jet_4_phi".to_owned() => ColumnType::Number,
			"jet_4_b_tag".to_owned() => ColumnType::Number,
			"m_jj".to_owned() => ColumnType::Number,
			"m_jjj".to_owned() => ColumnType::Number,
			"m_lv".to_owned() => ColumnType::Number,
			"m_jlv".to_owned() => ColumnType::Number,
			"m_bb".to_owned() => ColumnType::Number,
			"m_wbb".to_owned() => ColumnType::Number,
			"m_wwbb".to_owned() => ColumnType::Number,
		}),
		..Default::default()
	};
	let mut features = DataFrame::from_path(csv_file_path, options, |_| {}).unwrap();
	let labels = features.columns.remove(target_column_index);
	let (dataframe_train, dataframe_test) = features.view().split_at_row(nrows_train);
	let (labels_train, labels_test) = labels.view().split_at_row(nrows_train);
	let labels_train = labels_train.as_enum().unwrap();
	let labels_test = labels_test.as_enum().unwrap();

	// train the model
	let train_options = tangram_tree::TrainOptions {
		learning_rate: 0.1,
		max_depth: 8,
		max_leaf_nodes: 255,
		max_rounds: 100,
		min_examples_per_leaf: 100,
		min_sum_hessians_in_leaf: 0.0,
		..Default::default()
	};
	let model = tangram_tree::BinaryClassifier::train(
		dataframe_train,
		labels_train.clone(),
		train_options,
		&mut |_| {},
	);

	// make predictions on the test data
	let n_features = features.ncols();
	let columns = dataframe_test.columns;
	let mut features_ndarray = unsafe { Array2::uninitialized((nrows_test, n_features)) };
	izip!(features_ndarray.gencolumns_mut(), columns.as_slice()).for_each(
		|(mut feature_column, column)| {
			let column = column.as_number().unwrap();
			feature_column
				.iter_mut()
				.zip(column.data)
				.for_each(|(f, d)| *f = Value::Number(*d));
		},
	);
	let mut probabilities: Array2<f32> = unsafe { Array2::uninitialized((nrows_test, 2)) };
	model.predict(features_ndarray.view(), probabilities.view_mut());

	// compute metrics
	let mut metrics = tangram_metrics::ClassificationMetrics::new(2);
	metrics.update(tangram_metrics::ClassificationMetricsInput {
		probabilities: probabilities.view(),
		labels: labels_test.data.into(),
	});
	let metrics = metrics.finalize();
	println!("{:?}", metrics.accuracy);
}
