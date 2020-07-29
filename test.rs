use crate::{
	dataframe::*,
	features,
	gbt,
	linear,
	metrics,
	metrics::RunningMetric,
	// util::progress_counter::ProgressCounter,
};
use ndarray::prelude::*;
use rayon::prelude::*;

pub fn test_linear_regressor(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[features::FeatureGroup],
	model: &linear::Regressor,
	// progress: &ProgressCounter,
) -> metrics::RegressionMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = unsafe { Array2::uninitialized((dataframe_test.nrows(), n_features)) };
	features::compute_features_ndarray(dataframe_test, &feature_groups, features.view_mut());
	let labels: ArrayView1<f32> = dataframe_test
		.columns
		.get(target_column_index)
		.unwrap()
		.as_number()
		.unwrap()
		.data
		.into();
	let n_examples_per_batch = 256;
	struct State {
		predictions: Array1<f32>,
		metrics: metrics::RegressionMetrics,
	}
	let metrics = (
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		labels.axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
		.into_par_iter()
		.fold(
			|| {
				let predictions = unsafe { Array1::uninitialized(n_examples_per_batch) };
				let metrics = metrics::RegressionMetrics::default();
				State {
					predictions,
					metrics,
				}
			},
			|mut state, (features, labels)| {
				let slice = s![0..features.nrows()];
				model.predict(features, state.predictions.slice_mut(slice), None);
				state.metrics.update(metrics::RegressionMetricsInput {
					predictions: state.predictions.slice(slice),
					labels,
				});
				state
			},
		)
		.map(|state| state.metrics)
		.reduce(metrics::RegressionMetrics::default, |mut a, b| {
			a.merge(b);
			a
		});
	metrics.finalize()
}

pub fn test_gbt_regressor(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[features::FeatureGroup],
	model: &gbt::Regressor,
	// progress: &ProgressCounter,
) -> metrics::RegressionMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = unsafe { Array2::uninitialized((dataframe_test.nrows(), n_features)) };
	features::compute_features_ndarray_value(dataframe_test, feature_groups, features.view_mut());
	let labels: ArrayView1<f32> = dataframe_test
		.columns
		.get(target_column_index)
		.unwrap()
		.as_number()
		.unwrap()
		.data
		.into();
	let mut metrics = metrics::RegressionMetrics::default();
	let mut predictions = unsafe { Array1::uninitialized(features.nrows()) };
	model.predict(features.view(), predictions.view_mut(), None);
	metrics.update(metrics::RegressionMetricsInput {
		predictions: predictions.view(),
		labels,
	});
	metrics.finalize()
}

pub fn test_linear_binary_classifier(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[features::FeatureGroup],
	model: &linear::BinaryClassifier,
	// progress: &ProgressCounter,
) -> (
	metrics::ClassificationMetricsOutput,
	metrics::BinaryClassificationMetricsOutput,
) {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = unsafe { Array2::uninitialized((dataframe_test.nrows(), n_features)) };
	features::compute_features_ndarray(dataframe_test, &feature_groups, features.view_mut());
	let labels = dataframe_test
		.columns
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let n_classes = labels.options.len();
	let n_examples_per_batch = 256;
	struct State {
		predictions: Array2<f32>,
		classification_metrics: metrics::ClassificationMetrics,
		binary_classifier_metrics: metrics::BinaryClassifierMetrics,
	}
	let metrics = (
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.data).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
		.into_par_iter()
		.fold(
			|| {
				let predictions =
					unsafe { Array2::uninitialized((n_examples_per_batch, n_classes)) };
				State {
					predictions,
					classification_metrics: metrics::ClassificationMetrics::new(n_classes),
					binary_classifier_metrics: metrics::BinaryClassifierMetrics::new(100),
				}
			},
			|mut state, (features, labels)| {
				let slice = s![0..features.nrows(), ..];
				let predictions = state.predictions.slice_mut(slice);
				model.predict(features, predictions, None);
				let predictions = state.predictions.slice(slice);
				state
					.classification_metrics
					.update(metrics::ClassificationMetricsInput {
						probabilities: predictions,
						labels,
					});
				state
					.binary_classifier_metrics
					.update(metrics::BinaryClassifierMetricsInput {
						probabilities: predictions,
						labels,
					});
				state
			},
		)
		.map(|state| {
			(
				state.classification_metrics,
				state.binary_classifier_metrics,
			)
		})
		.reduce(
			|| {
				(
					metrics::ClassificationMetrics::new(n_classes),
					metrics::BinaryClassifierMetrics::new(100),
				)
			},
			|mut a, b| {
				a.0.merge(b.0);
				a.1.merge(b.1);
				a
			},
		);
	(metrics.0.finalize(), metrics.1.finalize())
}

pub fn test_gbt_binary_classifier(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[features::FeatureGroup],
	model: &gbt::BinaryClassifier,
	// progress: &ProgressCounter,
) -> (
	metrics::ClassificationMetricsOutput,
	metrics::BinaryClassificationMetricsOutput,
) {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = unsafe { Array2::uninitialized((dataframe_test.nrows(), n_features)) };
	features::compute_features_ndarray_value(dataframe_test, feature_groups, features.view_mut());
	let labels = dataframe_test
		.columns
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let n_classes = labels.options.len();
	let mut metrics = (
		metrics::ClassificationMetrics::new(n_classes),
		metrics::BinaryClassifierMetrics::new(100),
	);
	let mut predictions = unsafe { Array2::uninitialized((features.nrows(), n_classes)) };
	model.predict(features.view(), predictions.view_mut(), None);
	metrics.0.update(metrics::ClassificationMetricsInput {
		probabilities: predictions.view(),
		labels: labels.data.into(),
	});
	metrics.1.update(metrics::BinaryClassifierMetricsInput {
		probabilities: predictions.view(),
		labels: labels.data.into(),
	});
	(metrics.0.finalize(), metrics.1.finalize())
}

pub fn test_linear_multiclass_classifier(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[features::FeatureGroup],
	model: &linear::MulticlassClassifier,
	// progress: &ProgressCounter,
) -> metrics::ClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = unsafe { Array2::uninitialized((dataframe_test.nrows(), n_features)) };
	features::compute_features_ndarray(dataframe_test, &feature_groups, features.view_mut());
	let labels = dataframe_test
		.columns
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let n_classes = labels.options.len();
	let n_examples_per_batch = 256;
	struct State {
		predictions: Array2<f32>,
		metrics: metrics::ClassificationMetrics,
	}
	let metrics = (
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.data).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
		.into_par_iter()
		.fold(
			|| {
				let predictions =
					unsafe { Array2::uninitialized((n_examples_per_batch, n_classes)) };
				let metrics = metrics::ClassificationMetrics::new(n_classes);
				State {
					predictions,
					metrics,
				}
			},
			|mut state, (features, labels)| {
				let slice = s![0..features.nrows(), ..];
				let predictions = state.predictions.slice_mut(slice);
				model.predict(features, predictions, None);
				let predictions = state.predictions.slice(slice);
				state.metrics.update(metrics::ClassificationMetricsInput {
					probabilities: predictions,
					labels,
				});
				state
			},
		)
		.map(|state| state.metrics)
		.reduce(
			|| metrics::ClassificationMetrics::new(n_classes),
			|mut a, b| {
				a.merge(b);
				a
			},
		);
	metrics.finalize()
}

pub fn test_gbt_multiclass_classifier(
	dataframe_test: &DataFrameView,
	target_column_index: usize,
	feature_groups: &[features::FeatureGroup],
	model: &gbt::MulticlassClassifier,
	// progress: &ProgressCounter,
) -> metrics::ClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = unsafe { Array2::uninitialized((dataframe_test.nrows(), n_features)) };
	features::compute_features_ndarray_value(dataframe_test, feature_groups, features.view_mut());
	let labels = dataframe_test
		.columns
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let n_classes = labels.options.len();
	let mut metrics = metrics::ClassificationMetrics::new(n_classes);
	let mut predictions = unsafe { Array2::uninitialized((features.nrows(), n_classes)) };
	model.predict(features.view(), predictions.view_mut(), None);
	metrics.update(metrics::ClassificationMetricsInput {
		probabilities: predictions.view(),
		labels: labels.data.into(),
	});
	metrics.finalize()
}
