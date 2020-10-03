use super::{
	compute_shap_values_common, train_early_stopping_split, EarlyStoppingMonitor, TrainOptions,
	TrainProgress,
};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use rayon::prelude::*;
use std::{num::NonZeroUsize, ops::Neg};
use super_unsafe::SuperUnsafe;
use tangram_dataframe::*;
use tangram_metrics::{BinaryCrossEntropy, BinaryCrossEntropyInput, StreamingMetric};
use tangram_progress::ProgressCounter;
use tangram_thread_pool::pzip;

/// This struct describes a linear binary classifier model. You can train one by calling `BinaryClassifier::train`.
#[derive(Debug)]
pub struct BinaryClassifier {
	pub weights: Array1<f32>,
	pub bias: f32,
	/// These are the mean values of each feature in the training set, which are used to compute SHAP values.
	pub means: Vec<f32>,
	/// These are the loss values for each epoch.
	pub losses: Vec<f32>,
	/// These are the class names of the target column.
	pub classes: Vec<String>,
}

impl BinaryClassifier {
	/// Train a linear binary classifier.
	pub fn train(
		features: ArrayView2<f32>,
		labels: EnumColumnView,
		options: &TrainOptions,
		update_progress: &mut dyn FnMut(TrainProgress),
	) -> BinaryClassifier {
		let n_features = features.ncols();
		let classes: Vec<String> = labels.options.to_vec();
		let (features_train, labels_train, features_early_stopping, labels_early_stopping) =
			train_early_stopping_split(
				features,
				labels.data.into(),
				options
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
			losses: vec![],
			classes,
		};
		let mut early_stopping_monitor =
			if let Some(early_stopping_options) = &options.early_stopping_options {
				Some(EarlyStoppingMonitor::new(
					early_stopping_options.min_decrease_in_loss_for_significant_change,
					early_stopping_options.n_epochs_without_improvement_to_stop,
				))
			} else {
				None
			};
		let progress_counter = ProgressCounter::new(options.max_epochs.to_u64().unwrap());
		update_progress(TrainProgress(progress_counter.clone()));
		for _ in 0..options.max_epochs {
			progress_counter.inc(1);
			let model_cell = SuperUnsafe::new(model);
			izip!(
				features_train.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
				labels_train.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
			)
			.for_each(|(features, labels)| {
				let model = unsafe { model_cell.get() };
				Self::train_batch(model, features, labels, options);
			});
			model = model_cell.into_inner();
			if let Some(early_stopping_monitor) = early_stopping_monitor.as_mut() {
				let early_stopping_metric_value = Self::compute_early_stopping_metric_value(
					&model,
					features_early_stopping,
					labels_early_stopping,
					options,
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
		options: &TrainOptions,
	) {
		let learning_rate = options.learning_rate;
		let mut predictions = features.dot(&self.weights) + self.bias;
		for prediction in predictions.iter_mut() {
			*prediction = 1.0 / (prediction.neg().exp() + 1.0);
		}
		for (prediction, label) in izip!(predictions.view_mut(), labels) {
			let label = match label.map(|l| l.get()).unwrap_or(0) {
				1 => 0.0,
				2 => 1.0,
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
		options: &TrainOptions,
	) -> f32 {
		pzip!(
			features.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
			labels.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
		)
		.fold(
			|| {
				let predictions =
					unsafe { <Array2<f32>>::uninitialized((options.n_examples_per_batch, 2)) };
				let metric = BinaryCrossEntropy::new();
				(predictions, metric)
			},
			|(mut predictions, mut metric), (features, labels)| {
				let slice = s![0..features.nrows(), ..];
				let mut predictions_slice = predictions.slice_mut(slice);
				self.predict(features, predictions_slice.view_mut());
				for (prediction, label) in predictions_slice.column(1).iter().zip(labels.iter()) {
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
	pub fn predict(&self, features: ArrayView2<f32>, mut probabilities: ArrayViewMut2<f32>) {
		let mut probabilities_pos = probabilities.column_mut(1);
		probabilities_pos.fill(self.bias);
		ndarray::linalg::general_mat_vec_mul(
			1.0,
			&features,
			&self.weights,
			1.0,
			&mut probabilities_pos,
		);
		let (mut probabilities_neg, mut probabilities_pos) = probabilities.split_at(Axis(1), 1);
		for probability_pos in probabilities_pos.iter_mut() {
			*probability_pos = 1.0 / (probability_pos.neg().exp() + 1.0);
		}
		for (neg, pos) in izip!(probabilities_neg.view_mut(), probabilities_pos.view()) {
			*neg = 1.0 - *pos;
		}
	}

	/// Write SHAP values into `shap_values` for the input `features`.
	pub fn compute_shap_values(
		&self,
		features: ArrayView2<f32>,
		mut shap_values: ArrayViewMut3<f32>,
	) {
		for (features, mut shap_values) in izip!(
			features.axis_iter(Axis(0)),
			shap_values.axis_iter_mut(Axis(0)),
		) {
			compute_shap_values_common(
				features.as_slice().unwrap(),
				self.bias,
				self.weights.as_slice().unwrap(),
				&self.means,
				shap_values.row_mut(0).as_slice_mut().unwrap(),
			);
		}
	}
}
