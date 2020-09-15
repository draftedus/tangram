/*!
This crate is an implementation of linear machine learning models for regression and classification. There are three model types, [`Regressor`](struct.Regressor.html), [`BinaryClassifier`](struct.BinaryClassifier.html), and [`MulticlassClassifier`](struct.MulticlassClassifier.html). `BinaryClassifier` uses the sigmoid activation function, and `MulticlassClassifier` trains `n_classes` linear models whose outputs are combined with the `softmax` function.

To make training faster on multicore processors, we allow simultaneous read/write access to the model parameters from multiple threads. This means each thread will be reading weights partially updated by other threads and the weights it writes may be clobbered by other threads. Unsafe sharing is implmented using the [super_unsafe](docs.io/crates/super_unsafe) crate. This makes training nondeterministic, but in practice we observe little variation in the outcome, because there is feedback control: the change in loss is monitored after each epoch, and training terminates when the loss has stabilized.
*/

use ndarray::prelude::*;
use thiserror::Error;

mod binary_classifier;
mod early_stopping;
mod multiclass_classifier;
mod regressor;
mod shap;

pub use binary_classifier::BinaryClassifier;
pub use multiclass_classifier::MulticlassClassifier;
pub use regressor::Regressor;

/// These are the options passed to `Regressor::train`, `BinaryClassifier::train`, and `MulticlassClassifier::train`.
#[derive(Debug)]
pub struct TrainOptions {
	/// the percent of the dataset to set aside for computing early stopping metrics
	pub early_stopping_fraction: f32,
	/// the L2 regularization value to use when updating the model parameters
	pub l2_regularization: f32,
	/// the learning rate to use when updating the model parameters
	pub learning_rate: f32,
	/// the maximum number of epochs to train
	pub max_epochs: usize,
	/// the number of examples to use for each batch of training
	pub n_examples_per_batch: usize,
}

impl Default for TrainOptions {
	fn default() -> Self {
		Self {
			l2_regularization: 0.0,
			learning_rate: 0.1,
			max_epochs: 100,
			n_examples_per_batch: 128,
			early_stopping_fraction: 0.1,
		}
	}
}

/// the training progress, which tracks the current epoch
#[derive(Clone, Debug)]
pub struct Progress(pub tangram_progress::ProgressCounter);

#[derive(Debug, Error)]
pub enum Error {
	#[error("invalid label column")]
	InvalidLabelColumn,
}
