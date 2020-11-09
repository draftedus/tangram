use crate::train::ModelTestProgress;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use tangram_dataframe::prelude::*;
use tangram_metrics::{self as metrics, StreamingMetric};
use tangram_util::{progress_counter::ProgressCounter, zip};

pub fn test_linear_regressor(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[tangram_features::FeatureGroup],
	model: &tangram_linear::Regressor,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> metrics::RegressionMetricsOutput {
	let progress_counter = ProgressCounter::new(dataframe_test.ncols().to_u64().unwrap());
	update_progress(ModelTestProgress::ComputingFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_f32(dataframe_test, &feature_groups, &|| {
			progress_counter.inc(1)
		});
	update_progress(ModelTestProgress::Testing);
	let labels = dataframe_test.columns().get(target_column_index).unwrap();
	let labels = labels.as_number().unwrap();
	let n_examples_per_batch = 256;
	struct State {
		predictions: Array1<f32>,
		test_metrics: metrics::RegressionMetrics,
	}
	let State { test_metrics, .. } = zip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		labels.as_slice().chunks(n_examples_per_batch),
	)
	.fold(
		{
			let predictions = Array::zeros(n_examples_per_batch);
			let test_metrics = metrics::RegressionMetrics::default();
			State {
				predictions,
				test_metrics,
			}
		},
		|mut state, (features, labels)| {
			let slice = s![0..features.nrows()];
			model.predict(features, state.predictions.slice_mut(slice));
			state.test_metrics.update(metrics::RegressionMetricsInput {
				predictions: state.predictions.slice(slice).as_slice().unwrap(),
				labels,
			});
			state
		},
	);
	test_metrics.finalize()
}

pub fn test_tree_regressor(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[tangram_features::FeatureGroup],
	model: &tangram_tree::Regressor,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> metrics::RegressionMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_counter = ProgressCounter::new(n_features.to_u64().unwrap());
	update_progress(ModelTestProgress::ComputingFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_value(dataframe_test, feature_groups, &|| {
			progress_counter.inc(1)
		});
	let labels = dataframe_test.columns().get(target_column_index).unwrap();
	let labels = labels.as_number().unwrap();
	let mut test_metrics = metrics::RegressionMetrics::default();
	let mut predictions = Array::zeros(features.nrows());
	update_progress(ModelTestProgress::Testing);
	model.predict(features.view(), predictions.view_mut());
	test_metrics.update(metrics::RegressionMetricsInput {
		predictions: predictions.as_slice().unwrap(),
		labels: labels.as_slice(),
	});
	test_metrics.finalize()
}

pub fn test_linear_binary_classifier(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[tangram_features::FeatureGroup],
	model: &tangram_linear::BinaryClassifier,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> metrics::BinaryClassificationMetricsOutput {
	let progress_counter = ProgressCounter::new(dataframe_test.ncols().to_u64().unwrap());
	update_progress(ModelTestProgress::ComputingFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_f32(dataframe_test, &feature_groups, &|| {
			progress_counter.inc(1)
		});
	let labels = dataframe_test
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let n_examples_per_batch = 256;
	struct State {
		predictions: Array1<f32>,
		test_metrics: metrics::BinaryClassificationMetrics,
	}
	update_progress(ModelTestProgress::Testing);
	let State { test_metrics, .. } = zip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		{
			let predictions = Array::zeros(n_examples_per_batch);
			State {
				predictions,
				test_metrics: metrics::BinaryClassificationMetrics::new(101),
			}
		},
		|mut state, (features, labels)| {
			let slice = s![0..features.nrows()];
			let mut predictions = state.predictions.slice_mut(slice);
			model.predict(features, predictions.view_mut());
			state
				.test_metrics
				.update(metrics::BinaryClassificationMetricsInput {
					probabilities: predictions.as_slice().unwrap(),
					labels: labels.as_slice().unwrap(),
				});
			state
		},
	);
	test_metrics.finalize()
}

pub fn test_tree_binary_classifier(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[tangram_features::FeatureGroup],
	model: &tangram_tree::BinaryClassifier,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> metrics::BinaryClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_counter = ProgressCounter::new(n_features.to_u64().unwrap());
	update_progress(ModelTestProgress::ComputingFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_value(dataframe_test, feature_groups, &|| {
			progress_counter.inc(1)
		});
	let labels = dataframe_test
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let mut test_metrics = metrics::BinaryClassificationMetrics::new(101);
	let mut predictions = Array::zeros(features.nrows());
	update_progress(ModelTestProgress::Testing);
	model.predict(features.view(), predictions.view_mut());
	test_metrics.update(metrics::BinaryClassificationMetricsInput {
		probabilities: predictions.as_slice().unwrap(),
		labels: labels.as_slice(),
	});
	test_metrics.finalize()
}

pub fn test_linear_multiclass_classifier(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[tangram_features::FeatureGroup],
	model: &tangram_linear::MulticlassClassifier,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> metrics::MulticlassClassificationMetricsOutput {
	let progress_counter = ProgressCounter::new(dataframe_test.ncols().to_u64().unwrap());
	update_progress(ModelTestProgress::ComputingFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_f32(dataframe_test, &feature_groups, &|| {
			progress_counter.inc(1)
		});
	let labels = dataframe_test
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let n_classes = labels.options().len();
	let n_examples_per_batch = 256;
	struct State {
		predictions: Array2<f32>,
		test_metrics: metrics::MulticlassClassificationMetrics,
	}
	update_progress(ModelTestProgress::Testing);
	let State { test_metrics, .. } = zip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		{
			let predictions = Array::zeros((n_examples_per_batch, n_classes));
			let test_metrics = metrics::MulticlassClassificationMetrics::new(n_classes);
			State {
				predictions,
				test_metrics,
			}
		},
		|mut state, (features, labels)| {
			let slice = s![0..features.nrows(), ..];
			let predictions = state.predictions.slice_mut(slice);
			model.predict(features, predictions);
			let predictions = state.predictions.slice(slice);
			let labels = labels.view();
			state
				.test_metrics
				.update(metrics::MulticlassClassificationMetricsInput {
					probabilities: predictions,
					labels,
				});
			state
		},
	);
	test_metrics.finalize()
}

pub fn test_tree_multiclass_classifier(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[tangram_features::FeatureGroup],
	model: &tangram_tree::MulticlassClassifier,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> metrics::MulticlassClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_counter = ProgressCounter::new(n_features.to_u64().unwrap());
	update_progress(ModelTestProgress::ComputingFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_value(dataframe_test, feature_groups, &|| {
			progress_counter.inc(1)
		});
	let labels = dataframe_test
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let n_classes = labels.options().len();
	let mut test_metrics = metrics::MulticlassClassificationMetrics::new(n_classes);
	let mut predictions = Array::zeros((features.nrows(), n_classes));
	update_progress(ModelTestProgress::Testing);
	model.predict(features.view(), predictions.view_mut());
	test_metrics.update(metrics::MulticlassClassificationMetricsInput {
		probabilities: predictions.view(),
		labels: labels.as_slice().into(),
	});
	test_metrics.finalize()
}
