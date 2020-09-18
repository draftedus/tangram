/*!
This crate is an implementation of linear machine learning models for regression and classification. There are three model types, [`Regressor`](struct.Regressor.html), [`BinaryClassifier`](struct.BinaryClassifier.html), and [`MulticlassClassifier`](struct.MulticlassClassifier.html). `BinaryClassifier` uses the sigmoid activation function, and `MulticlassClassifier` trains `n_classes` linear models whose outputs are combined with the `softmax` function.

To make training faster on multicore processors, we allow simultaneous read/write access to the model parameters from multiple threads. This means each thread will be reading weights partially updated by other threads and the weights it writes may be clobbered by other threads. Unsafe sharing is implmented using the [super_unsafe](docs.io/crates/super_unsafe) crate. This makes training nondeterministic, but in practice we observe little variation in the outcome, because there is feedback control: the change in loss is monitored after each epoch, and training terminates when the loss has stabilized.
*/

use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;

mod binary_classifier;
mod multiclass_classifier;
mod regressor;

pub use binary_classifier::BinaryClassifier;
pub use multiclass_classifier::MulticlassClassifier;
pub use regressor::Regressor;

/// These are the options passed to `Regressor::train`, `BinaryClassifier::train`, and `MulticlassClassifier::train`.
#[derive(Debug)]
pub struct TrainOptions {
	/// Specify options for early stopping. If the value is `Some`, early stopping will be enabled. If it is `None`, early stopping will be disabled.
	pub early_stopping_options: Option<EarlyStoppingOptions>,
	/// This is the L2 regularization value to use when updating the model parameters.
	pub l2_regularization: f32,
	/// This is the learning rate to use when updating the model parameters.
	pub learning_rate: f32,
	/// This is the maximum number of epochs to train.
	pub max_epochs: usize,
	/// This is the number of examples to use for each batch of training.
	pub n_examples_per_batch: usize,
}

impl Default for TrainOptions {
	fn default() -> Self {
		Self {
			l2_regularization: 0.0,
			learning_rate: 0.1,
			max_epochs: 100,
			n_examples_per_batch: 128,
			early_stopping_options: Some(EarlyStoppingOptions {
				early_stopping_fraction: 0.1,
				n_epochs_without_improvement_to_stop: 3,
				min_decrease_in_loss_for_significant_change: 1e-3,
			}),
		}
	}
}

/// The parameters in this struct control how to determine whether training should stop early after each epoch.
#[derive(Debug)]
pub struct EarlyStoppingOptions {
	/// This is the fraction of the dataset that is set aside to compute the early stopping metric.
	pub early_stopping_fraction: f32,
	/// If this many epochs pass by without a significant improvement in the early stopping metric over the previous epoch, training will be stopped early.
	pub n_epochs_without_improvement_to_stop: usize,
	/// This is the minimum descrease in the early stopping metric for an epoch to be considered a significant improvement over the previous epoch.
	pub min_decrease_in_loss_for_significant_change: f32,
}

/// This is the training progress, which tracks the current epoch.
#[derive(Debug)]
pub struct TrainProgress(pub tangram_progress::ProgressCounter);

/// This function splits the `features` and `labels` arrays into training and early stopping arrays, where the size of the early stopping stopping array will be `features.len() * early_stopping_fraction`.
fn train_early_stopping_split<'features, 'labels, Label>(
	features: ArrayView2<'features, f32>,
	labels: ArrayView1<'labels, Label>,
	early_stopping_fraction: f32,
) -> (
	ArrayView2<'features, f32>,
	ArrayView1<'labels, Label>,
	ArrayView2<'features, f32>,
	ArrayView1<'labels, Label>,
) {
	let split_index = ((1.0 - early_stopping_fraction) * features.nrows().to_f32().unwrap())
		.to_usize()
		.unwrap();
	let (features_train, features_early_stopping) = features.split_at(Axis(0), split_index);
	let (labels_train, labels_early_stopping) = labels.split_at(Axis(0), split_index);
	(
		features_train,
		labels_train,
		features_early_stopping,
		labels_early_stopping,
	)
}

/**
The `EarlyStoppingMonitor` keeps track of the values of an early stopping metric for each epoch, and if enough epochs have passed without a significant improvement in the metric, the `update()` function will return `true` to indicate that training should be stopped.
*/
struct EarlyStoppingMonitor {
	threshold: f32,
	epochs: usize,
	n_epochs_without_observed_improvement: usize,
	previous_epoch_metric_value: Option<f32>,
}

impl EarlyStoppingMonitor {
	// Create a new `EarlyStoppingMonitor`.
	pub fn new(threshold: f32, epochs: usize) -> Self {
		EarlyStoppingMonitor {
			threshold,
			epochs,
			previous_epoch_metric_value: None,
			n_epochs_without_observed_improvement: 0,
		}
	}

	/// This function updates the `EarlyStoppingMonitor` with the next epoch's early stopping metric. THis function returns true if training should stop.
	pub fn update(&mut self, early_stopping_metric_value: f32) -> bool {
		let result = if let Some(previous_stopping_metric) = self.previous_epoch_metric_value {
			if early_stopping_metric_value > previous_stopping_metric
				|| f32::abs(early_stopping_metric_value - previous_stopping_metric) < self.threshold
			{
				self.n_epochs_without_observed_improvement += 1;
				self.n_epochs_without_observed_improvement >= self.epochs
			} else {
				self.n_epochs_without_observed_improvement = 0;
				false
			}
		} else {
			false
		};
		self.previous_epoch_metric_value = Some(early_stopping_metric_value);
		result
	}
}

/// This function is common code used by `compute_shap_values` for each model type.
fn compute_shap_values_common(
	example: ArrayView1<f32>,
	bias: f32,
	weights: ArrayView1<f32>,
	means: &[f32],
	shap_values: &mut [f32],
) {
	let mut bias_shap_value: f32 = bias;
	bias_shap_value += weights
		.iter()
		.zip(means.iter())
		.map(|(weight, mean)| weight * mean)
		.sum::<f32>();
	let len = shap_values.len();
	shap_values[len - 1] = bias_shap_value;
	for (shap_value, weight, feature, mean) in
		izip!(&mut shap_values[0..len - 1], weights, example, means)
	{
		*shap_value = weight * (feature - mean);
	}
}
