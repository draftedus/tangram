use crate::{features, train::ModelTestProgress};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use tangram_dataframe::prelude::*;
use tangram_metrics::{self as metrics, StreamingMetric};
use tangram_util::progress_counter::ProgressCounter;

pub fn test_linear_regressor(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[features::FeatureGroup],
	model: &tangram_linear::Regressor,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> metrics::RegressionMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_counter = ProgressCounter::new(n_features.to_u64().unwrap());
	update_progress(ModelTestProgress::ComputingFeatures(
		progress_counter.clone(),
	));
	let mut features = Array::zeros((dataframe_test.nrows(), n_features));
	features::compute_features_array_f32(
		dataframe_test,
		&feature_groups,
		features.view_mut(),
		&|| progress_counter.inc(1),
	);
	update_progress(ModelTestProgress::Testing);
	let labels = dataframe_test.columns().get(target_column_index).unwrap();
	let labels = labels.as_number().unwrap();
	let n_examples_per_batch = 256;
	struct State {
		predictions: Array1<f32>,
		metrics: metrics::RegressionMetrics,
	}
	let metrics = izip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		labels.as_slice().chunks(n_examples_per_batch),
	)
	.fold(
		{
			let predictions = Array::zeros(n_examples_per_batch);
			let metrics = metrics::RegressionMetrics::default();
			State {
				predictions,
				metrics,
			}
		},
		|mut state, (features, labels)| {
			let slice = s![0..features.nrows()];
			model.predict(features, state.predictions.slice_mut(slice));
			state.metrics.update(metrics::RegressionMetricsInput {
				predictions: state.predictions.slice(slice).as_slice().unwrap(),
				labels,
			});
			state
		},
	)
	.metrics;
	metrics.finalize()
}

pub fn test_tree_regressor(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[features::FeatureGroup],
	model: &tangram_tree::Regressor,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> metrics::RegressionMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = Array::from_elem(
		(dataframe_test.nrows(), n_features),
		DataFrameValue::Unknown,
	);
	let progress_counter = ProgressCounter::new(n_features.to_u64().unwrap());
	update_progress(ModelTestProgress::ComputingFeatures(
		progress_counter.clone(),
	));
	features::compute_features_array_value(
		dataframe_test,
		feature_groups,
		features.view_mut(),
		&|| progress_counter.inc(1),
	);
	let labels = dataframe_test.columns().get(target_column_index).unwrap();
	let labels = labels.as_number().unwrap();
	let mut metrics = metrics::RegressionMetrics::default();
	let mut predictions = Array::zeros(features.nrows());
	update_progress(ModelTestProgress::Testing);
	model.predict(features.view(), predictions.view_mut());
	metrics.update(metrics::RegressionMetricsInput {
		predictions: predictions.as_slice().unwrap(),
		labels: labels.as_slice(),
	});
	metrics.finalize()
}

pub fn test_linear_binary_classifier(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[features::FeatureGroup],
	model: &tangram_linear::BinaryClassifier,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> (
	metrics::MulticlassClassificationMetricsOutput,
	metrics::BinaryClassificationMetricsOutput,
) {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = Array::zeros((dataframe_test.nrows(), n_features));
	let progress_counter = ProgressCounter::new(n_features.to_u64().unwrap());
	update_progress(ModelTestProgress::ComputingFeatures(
		progress_counter.clone(),
	));
	features::compute_features_array_f32(
		dataframe_test,
		&feature_groups,
		features.view_mut(),
		&|| progress_counter.inc(1),
	);
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
		classification_metrics: metrics::MulticlassClassificationMetrics,
		binary_classifier_metrics: metrics::BinaryClassificationMetrics,
	}
	update_progress(ModelTestProgress::Testing);
	let metrics = izip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		{
			let predictions = Array::zeros((n_examples_per_batch, n_classes));
			State {
				predictions,
				classification_metrics: metrics::MulticlassClassificationMetrics::new(n_classes),
				binary_classifier_metrics: metrics::BinaryClassificationMetrics::new(100),
			}
		},
		|mut state, (features, labels)| {
			let slice = s![0..features.nrows(), ..];
			let predictions = state.predictions.slice_mut(slice);
			model.predict(features, predictions);
			let predictions = state.predictions.slice(slice);
			let labels = labels.view();
			state
				.classification_metrics
				.update(metrics::MulticlassClassificationMetricsInput {
					probabilities: predictions,
					labels,
				});
			state
				.binary_classifier_metrics
				.update(metrics::BinaryClassificationMetricsInput {
					probabilities: predictions,
					labels,
				});
			state
		},
	);
	(
		metrics.classification_metrics.finalize(),
		metrics.binary_classifier_metrics.finalize(),
	)
}

pub fn test_tree_binary_classifier(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[features::FeatureGroup],
	model: &tangram_tree::BinaryClassifier,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> (
	metrics::MulticlassClassificationMetricsOutput,
	metrics::BinaryClassificationMetricsOutput,
) {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_counter = ProgressCounter::new(n_features.to_u64().unwrap());
	update_progress(ModelTestProgress::ComputingFeatures(
		progress_counter.clone(),
	));
	let mut features = Array::from_elem(
		(dataframe_test.nrows(), n_features),
		DataFrameValue::Unknown,
	);
	features::compute_features_array_value(
		dataframe_test,
		feature_groups,
		features.view_mut(),
		&|| progress_counter.inc(1),
	);
	let labels = dataframe_test
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let n_classes = labels.options().len();
	let mut metrics = (
		metrics::MulticlassClassificationMetrics::new(n_classes),
		metrics::BinaryClassificationMetrics::new(100),
	);
	let mut predictions = Array::zeros((features.nrows(), n_classes));
	update_progress(ModelTestProgress::Testing);
	model.predict(features.view(), predictions.view_mut());
	metrics
		.0
		.update(metrics::MulticlassClassificationMetricsInput {
			probabilities: predictions.view(),
			labels: labels.as_slice().into(),
		});
	metrics.1.update(metrics::BinaryClassificationMetricsInput {
		probabilities: predictions.view(),
		labels: labels.as_slice().into(),
	});
	(metrics.0.finalize(), metrics.1.finalize())
}

pub fn test_linear_multiclass_classifier(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[features::FeatureGroup],
	model: &tangram_linear::MulticlassClassifier,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> metrics::MulticlassClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = Array::zeros((dataframe_test.nrows(), n_features));
	let progress_counter = ProgressCounter::new(n_features.to_u64().unwrap());
	update_progress(ModelTestProgress::ComputingFeatures(
		progress_counter.clone(),
	));
	features::compute_features_array_f32(
		dataframe_test,
		&feature_groups,
		features.view_mut(),
		&|| progress_counter.inc(1),
	);
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
		metrics: metrics::MulticlassClassificationMetrics,
	}
	update_progress(ModelTestProgress::Testing);
	let metrics = izip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		{
			let predictions = Array::zeros((n_examples_per_batch, n_classes));
			let metrics = metrics::MulticlassClassificationMetrics::new(n_classes);
			State {
				predictions,
				metrics,
			}
		},
		|mut state, (features, labels)| {
			let slice = s![0..features.nrows(), ..];
			let predictions = state.predictions.slice_mut(slice);
			model.predict(features, predictions);
			let predictions = state.predictions.slice(slice);
			let labels = labels.view();
			state
				.metrics
				.update(metrics::MulticlassClassificationMetricsInput {
					probabilities: predictions,
					labels,
				});
			state
		},
	)
	.metrics;
	metrics.finalize()
}

pub fn test_tree_multiclass_classifier(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[features::FeatureGroup],
	model: &tangram_tree::MulticlassClassifier,
	update_progress: &mut dyn FnMut(ModelTestProgress),
) -> metrics::MulticlassClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = Array::from_elem(
		(dataframe_test.nrows(), n_features),
		DataFrameValue::Unknown,
	);
	let progress_counter = ProgressCounter::new(n_features.to_u64().unwrap());
	update_progress(ModelTestProgress::ComputingFeatures(
		progress_counter.clone(),
	));
	features::compute_features_array_value(
		dataframe_test,
		feature_groups,
		features.view_mut(),
		&|| progress_counter.inc(1),
	);
	let labels = dataframe_test
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let n_classes = labels.options().len();
	let mut metrics = metrics::MulticlassClassificationMetrics::new(n_classes);
	let mut predictions = Array::zeros((features.nrows(), n_classes));
	update_progress(ModelTestProgress::Testing);
	model.predict(features.view(), predictions.view_mut());
	metrics.update(metrics::MulticlassClassificationMetricsInput {
		probabilities: predictions.view(),
		labels: labels.as_slice().into(),
	});
	metrics.finalize()
}
