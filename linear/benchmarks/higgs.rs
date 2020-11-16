use maplit::btreemap;
use ndarray::prelude::*;
use rayon::prelude::*;
use serde_json::json;
use std::path::Path;
use tangram_dataframe::prelude::*;
use tangram_metrics::Metric;
use tangram_util::{pzip, zip};

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/higgs_train.csv");
	let csv_file_path_test = Path::new("data/higgs_test.csv");
	let target_column_index = 0;
	let options = tangram_dataframe::FromCsvOptions {
		column_types: Some(btreemap! {
			"signal".to_owned() => DataFrameColumnType::Enum { options: vec!["false".to_owned(), "true".to_owned()] },
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
	let labels_train = labels_train.as_enum().unwrap();
	let mut features_test =
		DataFrame::from_path(csv_file_path_test, options.clone(), |_| {}).unwrap();
	let labels_test = features_test.columns_mut().remove(target_column_index);
	let labels_test = labels_test.as_enum().unwrap();
	let feature_groups: Vec<tangram_features::FeatureGroup> = features_train
		.columns()
		.iter()
		.map(|column| match column {
			DataFrameColumn::Number(column) => {
				let mean_variance =
					tangram_metrics::MeanVariance::compute(column.view().as_slice());
				tangram_features::FeatureGroup::Normalized(
					tangram_features::NormalizedFeatureGroup {
						source_column_name: column.name().clone().unwrap(),
						mean: mean_variance.mean,
						variance: mean_variance.variance,
					},
				)
			}
			_ => unreachable!(),
		})
		.collect();
	let features_train = tangram_features::compute_features_array_f32(
		&features_train.view(),
		feature_groups.as_slice(),
		&|| {},
	);
	let features_test = tangram_features::compute_features_array_f32(
		&features_test.view(),
		feature_groups.as_slice(),
		&|| {},
	);

	// Train the model.
	let train_output = tangram_linear::BinaryClassifier::train(
		features_train.view(),
		labels_train.view(),
		&tangram_linear::TrainOptions {
			learning_rate: 0.01,
			max_epochs: 1,
			n_examples_per_batch: 1000,
			..Default::default()
		},
		&mut |_| {},
	);

	// Make predictions on the test data.
	let chunk_size =
		(features_test.nrows() + rayon::current_num_threads() - 1) / rayon::current_num_threads();
	let mut probabilities = Array::zeros(features_test.nrows());
	pzip!(
		features_test.axis_chunks_iter(Axis(0), chunk_size),
		probabilities.axis_chunks_iter_mut(Axis(0), chunk_size),
	)
	.for_each(|(features_test_chunk, probabilities_chunk)| {
		train_output
			.model
			.predict(features_test_chunk, probabilities_chunk);
	});

	// Compute metrics.
	let input = zip!(probabilities.iter(), labels_test.iter())
		.map(|(probability, label)| (*probability, label.unwrap()))
		.collect();
	let auc_roc = tangram_metrics::AUCROC::compute(input);

	// Compute memory usage.
	let mut memory = None;
	let file = std::fs::read_to_string("/proc/self/status").unwrap();
	for line in file.lines() {
		if line.starts_with("VmHWM") {
			memory = Some(line.split(':').nth(1).map(|x| x.trim().to_owned()).unwrap());
		}
	}

	let output = json!({
		"auc_roc": auc_roc,
		"memory": memory,
	});
	println!("{}", output);
}
