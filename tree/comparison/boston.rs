use itertools::izip;
use ndarray::prelude::*;
use std::path::Path;
use tangram::{dataframe::*, metrics::Metric};

fn main() {
	// load the data
	let csv_file_path = Path::new("data/boston.csv");
	let nrows_train = 405;
	let _nrows_test = 101;

	let target_column_index = 13;
	let mut csv_reader = csv::Reader::from_path(csv_file_path).unwrap();
	let options = FromCsvOptions {
		..Default::default()
	};
	let mut features = DataFrame::from_csv(&mut csv_reader, options, |_| {}).unwrap();
	let labels = features.columns.remove(target_column_index);
	let (features_train, features_test) = features.view().split_at_row(nrows_train);
	let (labels_train, labels_test) = labels.view().split_at_row(nrows_train);
	let labels_train = labels_train.as_number().unwrap();
	let labels_test = labels_test.as_number().unwrap();

	// train the model
	let train_options = tangram::tree::TrainOptions {
		learning_rate: 0.1,
		max_depth: 8,
		max_leaf_nodes: 255,
		max_rounds: 100,
		min_examples_leaf: 100,
		min_sum_hessians_in_leaf: 0.0,
		..Default::default()
	};
	let model = tangram::tree::Regressor::train(
		features_train,
		labels_train.clone(),
		train_options,
		&mut |_| {},
	);

	// make predictions on the test data
	let n_features = features.ncols();
	let nrows = features_test.nrows();
	let columns = features_test.columns;
	let mut features_ndarray = unsafe { Array2::uninitialized((nrows, n_features)) };
	izip!(features_ndarray.gencolumns_mut(), columns.as_slice()).for_each(
		|(mut feature_column, column)| match column {
			ColumnView::Number(column) => {
				feature_column
					.iter_mut()
					.zip(column.data)
					.for_each(|(f, d)| *f = Value::Number(*d));
			}
			ColumnView::Enum(column) => {
				feature_column
					.iter_mut()
					.zip(column.data)
					.for_each(|(f, d)| *f = Value::Enum(*d));
			}
			_ => panic!(),
		},
	);
	let mut predictions: Array1<f32> = unsafe { Array::uninitialized(nrows) };
	model.predict(features_ndarray.view(), predictions.view_mut(), None);

	// compute metrics
	let mut metrics = tangram::metrics::RegressionMetrics::new();
	metrics.update(tangram::metrics::RegressionMetricsInput {
		predictions: predictions.as_slice().unwrap(),
		labels: labels_test.data,
	});
	let metrics = metrics.finalize();
	println!("{:?}", metrics);
}
