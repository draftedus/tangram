use anyhow::Result;
use maplit::btreemap;
use ndarray::{prelude::*, Zip};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::path::Path;
use std::time::Instant;
use tangram_core::dataframe::*;
use tangram_core::metrics;

fn main() -> Result<()> {
	// load the data
	let csv_file_path = Path::new("data/higgs.csv");
	let nrows_train = 10_500_000;
	let nrows_test = 500_000;
	let target_column_index = 0;
	let mut csv_reader = csv::Reader::from_path(csv_file_path)?;
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
	let mut features = DataFrame::from_csv(&mut csv_reader, options, |_| {})?;
	let labels = features.columns.remove(target_column_index);
	let (dataframe_train, dataframe_test) = features.view().split_at_row(nrows_train);
	let (labels_train, labels_test) = labels.view().split_at_row(nrows_train);
	let labels_train = labels_train.as_enum().unwrap();
	let labels_test = labels_test.as_enum().unwrap();

	// train the model
	let train_options = tangram_core::gbt::TrainOptions {
		learning_rate: 0.1,
		max_depth: 8,
		max_leaf_nodes: 255,
		max_rounds: 100,
		min_examples_leaf: 100,
		min_sum_hessians_in_leaf: 0.0,
		..Default::default()
	};
	let start = Instant::now();
	let model = tangram_core::gbt::BinaryClassifier::train(
		dataframe_train,
		labels_train.clone(),
		train_options,
		&mut |_| {},
	);
	let end = Instant::now();
	println!("duration: {:?}", end - start);

	let n_features = features.ncols();
	let columns = dataframe_test.columns;
	let mut features_ndarray = unsafe { Array2::uninitialized((nrows_test, n_features)) };
	Zip::from(features_ndarray.gencolumns_mut())
		.and(columns.as_slice())
		.into_par_iter()
		.for_each(|(mut feature_column, column)| {
			let column = column.as_number().unwrap();
			feature_column
				.iter_mut()
				.zip(column.data)
				.for_each(|(f, d)| *f = Value::Number(*d));
		});

	let mut probabilities: Array2<f32> = unsafe { Array::uninitialized((nrows_test, 2)) };
	model.predict(features_ndarray.view(), probabilities.view_mut(), None);
	let accuracy = metrics::accuracy(probabilities.view(), labels_test.data.into());
	println!("accuracy: {:?}", accuracy);

	Ok(())
}
