use super::{
	shap::{compute_shap_values_for_example, ComputeShapValuesForExampleOutput},
	train_early_stopping_split, EarlyStoppingMonitor, TrainOptions, TrainProgress,
};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use rayon::prelude::*;
use std::{num::NonZeroUsize, ops::Neg};
use tangram_dataframe::prelude::*;
use tangram_metrics::{BinaryCrossEntropy, BinaryCrossEntropyInput, StreamingMetric};
use tangram_util::{progress_counter::ProgressCounter, pzip, super_unsafe::SuperUnsafe};

/// This struct describes a linear binary classifier model. You can train one by calling `BinaryClassifier::train`.
#[derive(Debug)]
pub struct BinaryClassifier {
	/// These are the weights the model learned.
	pub weights: Array1<f32>,
	/// This is the bias the model learned.
	pub bias: f32,
	/// These are the mean values of each feature in the training set, which are used to compute SHAP values.
	pub means: Vec<f32>,
}

impl BinaryClassifier {
	/// Train a linear binary classifier.
	pub fn train(
		features: ArrayView2<f32>,
		labels: EnumDataFrameColumnView,
		train_options: &TrainOptions,
		update_progress: &mut dyn FnMut(TrainProgress),
	) -> BinaryClassifier {
		let n_features = features.ncols();
		let (features_train, labels_train, features_early_stopping, labels_early_stopping) =
			train_early_stopping_split(
				features,
				labels.as_slice().into(),
				train_options
					.early_stopping_options
					.as_ref()
					.map(|o| o.early_stopping_fraction)
					.unwrap_or(0.0),
			);
		let means = features_train
			.axis_iter(Axis(1))
			.map(|column| column.mean().unwrap())
			.collect();
		let mut model = BinaryClassifier {
			bias: 0.0,
			weights: <Array1<f32>>::zeros(n_features),
			means,
		};
		let mut early_stopping_monitor =
			if let Some(early_stopping_options) = &train_options.early_stopping_options {
				Some(EarlyStoppingMonitor::new(
					early_stopping_options.min_decrease_in_loss_for_significant_change,
					early_stopping_options.n_epochs_without_improvement_to_stop,
				))
			} else {
				None
			};
		let progress_counter = ProgressCounter::new(train_options.max_epochs.to_u64().unwrap());
		update_progress(TrainProgress(progress_counter.clone()));
		for _ in 0..train_options.max_epochs {
			progress_counter.inc(1);
			let n_examples_per_batch = train_options.n_examples_per_batch;
			let model_cell = SuperUnsafe::new(model);
			pzip!(
				features_train.axis_chunks_iter(Axis(0), n_examples_per_batch),
				labels_train.axis_chunks_iter(Axis(0), n_examples_per_batch),
			)
			.for_each(|(features, labels)| {
				let model = unsafe { model_cell.get() };
				BinaryClassifier::train_batch(model, features, labels, train_options);
			});
			model = model_cell.into_inner();
			if let Some(early_stopping_monitor) = early_stopping_monitor.as_mut() {
				let early_stopping_metric_value =
					BinaryClassifier::compute_early_stopping_metric_value(
						&model,
						features_early_stopping,
						labels_early_stopping,
						train_options,
					);
				let should_stop = early_stopping_monitor.update(early_stopping_metric_value);
				if should_stop {
					break;
				}
			}
		}
		model
	}

	fn train_batch(
		&mut self,
		features: ArrayView2<f32>,
		labels: ArrayView1<Option<NonZeroUsize>>,
		train_options: &TrainOptions,
	) {
		let learning_rate = train_options.learning_rate;
		let mut predictions = features.dot(&self.weights) + self.bias;
		for prediction in predictions.iter_mut() {
			*prediction = 1.0 / (prediction.neg().exp() + 1.0);
		}
		for (prediction, label) in izip!(predictions.view_mut(), labels) {
			let label = match label.map(|l| l.get()) {
				Some(1) => 0.0,
				Some(2) => 1.0,
				_ => unreachable!(),
			};
			*prediction -= label
		}
		let py = predictions.insert_axis(Axis(1));
		let weight_gradients = (&features * &py).mean_axis(Axis(0)).unwrap();
		let bias_gradient = py.mean_axis(Axis(0)).unwrap()[0];
		for (weight, weight_gradient) in izip!(self.weights.view_mut(), weight_gradients.view()) {
			*weight += -learning_rate * weight_gradient;
		}
		self.bias += -learning_rate * bias_gradient;
	}

	fn compute_early_stopping_metric_value(
		&self,
		features: ArrayView2<f32>,
		labels: ArrayView1<Option<NonZeroUsize>>,
		train_options: &TrainOptions,
	) -> f32 {
		pzip!(
			features.axis_chunks_iter(Axis(0), train_options.n_examples_per_batch),
			labels.axis_chunks_iter(Axis(0), train_options.n_examples_per_batch),
		)
		.fold(
			|| {
				let predictions =
					unsafe { <Array1<f32>>::uninitialized(train_options.n_examples_per_batch) };
				let metric = BinaryCrossEntropy::new();
				(predictions, metric)
			},
			|(mut predictions, mut metric), (features, labels)| {
				let slice = s![0..features.nrows()];
				let mut predictions_slice = predictions.slice_mut(slice);
				self.predict(features, predictions_slice.view_mut());
				for (prediction, label) in izip!(predictions_slice.iter(), labels.iter()) {
					metric.update(BinaryCrossEntropyInput {
						probability: *prediction,
						label: *label,
					});
				}
				(predictions, metric)
			},
		)
		.map(|(_, metric)| metric)
		.reduce(BinaryCrossEntropy::new, |mut a, b| {
			a.merge(b);
			a
		})
		.finalize()
		.unwrap()
	}

	/// Write predicted probabilities into `probabilities` for the input `features`.
	pub fn predict(&self, features: ArrayView2<f32>, mut probabilities: ArrayViewMut1<f32>) {
		probabilities.fill(self.bias);
		ndarray::linalg::general_mat_vec_mul(
			1.0,
			&features,
			&self.weights,
			1.0,
			&mut probabilities,
		);
		for probability in probabilities.iter_mut() {
			*probability = 1.0 / (probability.neg().exp() + 1.0);
		}
	}

	pub fn compute_feature_contributions(
		&self,
		features: ArrayView2<f32>,
	) -> Vec<ComputeShapValuesForExampleOutput> {
		features
			.axis_iter(Axis(0))
			.map(|features| {
				compute_shap_values_for_example(
					features.as_slice().unwrap(),
					self.bias,
					self.weights.as_slice().unwrap(),
					&self.means,
				)
			})
			.collect()
	}
}
