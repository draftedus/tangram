use maplit::btreemap;
use ndarray::prelude::*;
use rayon::prelude::*;
use std::path::Path;
use tangram_dataframe::prelude::*;
use tangram_metrics::StreamingMetric;
use tangram_thread_pool::pzip;

fn main() {
	// Load the data.
	// let csv_file_path = Path::new("data/higgs.csv");
	// let (nrows_train, _) = (10_500_000, 500_000);
	let csv_file_path = Path::new("data/higgs-small.csv");
	let (nrows_train, _) = (450_000, 50_000);
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
	let mut features = DataFrame::from_path(csv_file_path, options, |_| {}).unwrap();
	let labels = features.columns.remove(target_column_index);
	let (features_train, features_test) = features.view().split_at_row(nrows_train);
	let (labels_train, labels_test) = labels.view().split_at_row(nrows_train);
	let labels_train = labels_train.as_enum().unwrap();
	let labels_test = labels_test.as_enum().unwrap();

	// Train the model.
	let start = std::time::Instant::now();
	let train_options = tangram_tree::TrainOptions {
		binned_features_layout: tangram_tree::BinnedFeaturesLayout::ColumnMajor,
		learning_rate: 0.1,
		max_leaf_nodes: 255,
		max_rounds: 100,
		..Default::default()
	};
	let model = tangram_tree::BinaryClassifier::train(
		features_train,
		labels_train.clone(),
		train_options,
		&mut |_| {},
	);
	let duration = start.elapsed();

	// Make predictions on the test data and compute metrics.
	let features_test = features_test.to_rows();
	let chunk_size = features_test.nrows() / rayon::current_num_threads();
	let metrics = pzip!(
		features_test.axis_chunks_iter(Axis(0), chunk_size),
		labels_test.data.par_chunks(chunk_size),
	)
	.fold(
		|| {
			let metrics = tangram_metrics::BinaryClassificationMetrics::new(100);
			let probabilities: Array2<f32> = unsafe { Array2::uninitialized((chunk_size, 2)) };
			(metrics, probabilities)
		},
		|(mut metrics, mut probabilities), (features_test, labels_test)| {
			let probabilities_slice = s![0..features_test.nrows(), ..];
			model.predict(features_test, probabilities.slice_mut(probabilities_slice));
			metrics.update(tangram_metrics::BinaryClassificationMetricsInput {
				probabilities: probabilities.slice(probabilities_slice),
				labels: labels_test.into(),
			});
			(metrics, probabilities)
		},
	)
	.map(|(metrics, _)| metrics)
	.reduce(
		|| tangram_metrics::BinaryClassificationMetrics::new(100),
		|mut a, b| {
			a.merge(b);
			a
		},
	)
	.finalize();

	// Print the results.
	println!("duration {}", duration.as_secs_f32());
	println!("accuracy {}", metrics.thresholds[50].accuracy);
	println!("auc {}", metrics.auc_roc);
}
