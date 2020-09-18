use itertools::izip;
use ndarray::prelude::*;
use std::path::Path;
use tangram_dataframe::*;
use tangram_metrics::StreamingMetric;

fn main() {
	// load the data
	let csv_file_path = Path::new("data/heart-disease.csv");
	let nrows_train = 242;
	let _nrows_test = 61;
	let target_column_index = 13;
	let options = FromCsvOptions {
		..Default::default()
	};
	let mut features = DataFrame::from_path(csv_file_path, options, |_| {}).unwrap();

	let labels = features.columns.remove(target_column_index);
	let (features_train, features_test) = features.view().split_at_row(nrows_train);
	let (labels_train, labels_test) = labels.view().split_at_row(nrows_train);
	let labels_train = labels_train.as_enum().unwrap();
	let labels_test = labels_test.as_enum().unwrap();

	// train the model
	let train_options = tangram_tree::TrainOptions {
		learning_rate: 0.1,
		max_depth: 8,
		max_leaf_nodes: 255,
		max_rounds: 100,
		min_examples_per_leaf: 10,
		min_sum_hessians_in_leaf: 0.0,
		..Default::default()
	};
	let model = tangram_tree::BinaryClassifier::train(
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
	let mut probabilities: Array2<f32> = unsafe { Array::uninitialized((nrows, 2)) };
	model.predict(features_ndarray.view(), probabilities.view_mut());

	// compute metrics
	let mut metrics = tangram_metrics::BinaryClassificationMetrics::new(100);
	metrics.update(tangram_metrics::BinaryClassificationMetricsInput {
		probabilities: probabilities.view(),
		labels: labels_test.data.into(),
	});
	let metrics = metrics.finalize();
	println!("{:?}", metrics);
}
