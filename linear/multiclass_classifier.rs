use super::{
	shap::{compute_shap_values_for_example, ComputeShapValuesForExampleOutput},
	train_early_stopping_split, EarlyStoppingMonitor, TrainOptions, TrainProgress,
};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use rayon::prelude::*;
use std::num::NonZeroUsize;
use super_unsafe::SuperUnsafe;
use tangram_dataframe::prelude::*;
use tangram_metrics::{CrossEntropy, CrossEntropyInput, StreamingMetric};
use tangram_progress::ProgressCounter;
use tangram_thread_pool::pzip;

/// This struct describes a linear multiclass classifier model. You can train one by calling `MulticlassClassifier::train`.
#[derive(Debug)]
pub struct MulticlassClassifier {
	pub weights: Array2<f32>,
	pub biases: Array1<f32>,
	/// These are the mean values of each feature in the training set, which are used to compute SHAP values.
	pub means: Vec<f32>,
	/// These are the loss values for each epoch.
	pub losses: Option<Vec<f32>>,
	/// These are the class names of the target column.
	pub classes: Vec<String>,
}

impl MulticlassClassifier {
	/// Train a linear multiclass classifier.
	pub fn train(
		features: ArrayView2<f32>,
		labels: EnumDataFrameColumnView,
		options: &TrainOptions,
		update_progress: &mut dyn FnMut(TrainProgress),
	) -> MulticlassClassifier {
		let n_classes = labels.options.len();
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
		let mut model = MulticlassClassifier {
			biases: <Array1<f32>>::zeros(n_classes),
			weights: <Array2<f32>>::zeros((n_features, n_classes)),
			means,
			losses: None,
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
			pzip!(
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
		let n_classes = self.weights.ncols();
		let mut logits = features.dot(&self.weights) + &self.biases;
		softmax(logits.view_mut());
		let mut predictions = logits;
		for (mut predictions, label) in izip!(predictions.axis_iter_mut(Axis(0)), labels) {
			for (class_index, prediction) in predictions.iter_mut().enumerate() {
				*prediction -= if class_index == label.unwrap().get() - 1 {
					1.0
				} else {
					0.0
				};
			}
		}
		let py = predictions;
		for class_index in 0..n_classes {
			let weight_gradients = (&features * &py.column(class_index).insert_axis(Axis(1)))
				.mean_axis(Axis(0))
				.unwrap();
			for (weight, weight_gradient) in izip!(
				self.weights.column_mut(class_index),
				weight_gradients.iter()
			) {
				*weight += -learning_rate * weight_gradient
			}
			let bias_gradients = py
				.column(class_index)
				.insert_axis(Axis(1))
				.mean_axis(Axis(0))
				.unwrap();
			self.biases[class_index] += -learning_rate * bias_gradients[0];
		}
	}

	fn compute_early_stopping_metric_value(
		&self,
		features: ArrayView2<f32>,
		labels: ArrayView1<Option<NonZeroUsize>>,
		options: &TrainOptions,
	) -> f32 {
		let n_classes = self.biases.len();
		pzip!(
			features.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
			labels.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
		)
		.fold(
			|| {
				let predictions = unsafe {
					<Array2<f32>>::uninitialized((options.n_examples_per_batch, n_classes))
				};
				let metric = CrossEntropy::default();
				(predictions, metric)
			},
			|(mut predictions, mut metric), (features, labels)| {
				let slice = s![0..features.nrows(), ..];
				let mut predictions_slice = predictions.slice_mut(slice);
				self.predict(features, predictions_slice.view_mut());
				for (prediction, label) in
					izip!(predictions_slice.axis_iter(Axis(0)), labels.iter())
				{
					metric.update(CrossEntropyInput {
						probabilities: prediction,
						label: *label,
					});
				}
				(predictions, metric)
			},
		)
		.map(|(_, metric)| metric)
		.reduce(CrossEntropy::new, |mut a, b| {
			a.merge(b);
			a
		})
		.finalize()
		.0
		.unwrap()
	}

	/// Write predicted probabilities into `probabilities` for the input `features`.
	pub fn predict(&self, features: ArrayView2<f32>, mut probabilities: ArrayViewMut2<f32>) {
		for mut row in probabilities.axis_iter_mut(Axis(0)) {
			row.assign(&self.biases.view());
		}
		ndarray::linalg::general_mat_mul(1.0, &features, &self.weights, 1.0, &mut probabilities);
		softmax(probabilities);
	}

	pub fn compute_feature_contributions(
		&self,
		features: ArrayView2<f32>,
	) -> Vec<Vec<ComputeShapValuesForExampleOutput>> {
		features
			.axis_iter(Axis(0))
			.map(|features| {
				izip!(self.weights.axis_iter(Axis(0)), self.biases.view())
					.map(|(weights, bias)| {
						compute_shap_values_for_example(
							features.as_slice().unwrap(),
							*bias,
							weights.as_slice().unwrap(),
							&self.means,
						)
					})
					.collect()
			})
			.collect()
	}
}

fn softmax(mut logits: ArrayViewMut2<f32>) {
	for mut logits in logits.axis_iter_mut(Axis(0)) {
		let max = logits.iter().fold(std::f32::MIN, |a, &b| f32::max(a, b));
		for logit in logits.iter_mut() {
			*logit = (*logit - max).exp();
		}
		let sum = logits.iter().sum::<f32>();
		for logit in logits.iter_mut() {
			*logit /= sum;
		}
	}
}
