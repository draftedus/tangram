use maplit::btreemap;
use ndarray::prelude::*;
use rayon::prelude::*;
use std::path::Path;
use tangram_dataframe::prelude::*;
use tangram_metrics::Metric;
use tangram_util::pzip;

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/higgs_train.csv");
	let csv_file_path_test = Path::new("data/higgs_test.csv");
	let _nrows_train = 10_500_000;
	let _nrows_test = 500_000;
	let target_column_index = 0;
	let options = tangram_dataframe::FromCsvOptions {
		column_types: Some(btreemap! {
			"signal".to_owned() => DataFrameColumnType::Enum { options: vec!["false".into(), "true".into()] },
			"lepton_pt".to_owned() => DataFrameColumnType::Number,
			"lepton_eta".to_owned() => DataFrameColumnType::Number,
			"lepton_phi".to_owned() => DataFrameColumnType::Number,
			"missing_energy_magnitude".to_owned() => DataFrameColumnType::Number,
			"missing_energy_phi".to_owned() => DataFrameColumnType::Number,
			"jet_1_pt".to_owned() => DataFrameColumnType::Number,
			"jet_1_eta".to_owned() => DataFrameColumnType::Number,
			"jet_1_phi".to_owned() => DataFrameColumnType::Number,
			"jet_1_b_tag".to_owned() => DataFrameColumnType::Number,
			"jet_2_pt".to_owned() => DataFrameColumnType::Number,
			"jet_2_eta".to_owned() => DataFrameColumnType::Number,
			"jet_2_phi".to_owned() => DataFrameColumnType::Number,
			"jet_2_b_tag".to_owned() => DataFrameColumnType::Number,
			"jet_3_pt".to_owned() => DataFrameColumnType::Number,
			"jet_3_eta".to_owned() => DataFrameColumnType::Number,
			"jet_3_phi".to_owned() => DataFrameColumnType::Number,
			"jet_3_b_tag".to_owned() => DataFrameColumnType::Number,
			"jet_4_pt".to_owned() => DataFrameColumnType::Number,
			"jet_4_eta".to_owned() => DataFrameColumnType::Number,
			"jet_4_phi".to_owned() => DataFrameColumnType::Number,
			"jet_4_b_tag".to_owned() => DataFrameColumnType::Number,
			"m_jj".to_owned() => DataFrameColumnType::Number,
			"m_jjj".to_owned() => DataFrameColumnType::Number,
			"m_lv".to_owned() => DataFrameColumnType::Number,
			"m_jlv".to_owned() => DataFrameColumnType::Number,
			"m_bb".to_owned() => DataFrameColumnType::Number,
			"m_wbb".to_owned() => DataFrameColumnType::Number,
			"m_wwbb".to_owned() => DataFrameColumnType::Number,
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
	let start = std::time::Instant::now();
	let train_options = tangram_tree::TrainOptions {
		binned_features_layout: tangram_tree::BinnedFeaturesLayout::RowMajor,
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
	println!("duration {}", start.elapsed().as_secs_f32());

	// Make predictions on the test data.
	let features_test = features_test.to_rows();
	let chunk_size =
		(features_test.nrows() + rayon::current_num_threads() - 1) / rayon::current_num_threads();
	let mut probabilities = Array::zeros((features_test.nrows(), 2));
	pzip!(
		features_test.axis_chunks_iter(Axis(0), chunk_size),
		probabilities.axis_chunks_iter_mut(Axis(0), chunk_size),
	)
	.for_each(|(features_test_chunk, probabilities_chunk)| {
		model.predict(features_test_chunk, probabilities_chunk);
	});

	// Compute metrics.
	let labels = labels_test;
	let input = probabilities
		.column(1)
		.iter()
		.cloned()
		.zip(labels.iter().map(|d| d.unwrap()))
		.collect();
	let auc_roc = tangram_metrics::AUCROC::compute(input);
	println!("auc {}", auc_roc);
}
