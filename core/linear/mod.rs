/*!
This module implements linear models for regression and classification.
*/

mod binary_classifier;
mod early_stopping;
mod multiclass_classifier;
mod progress;
mod regressor;
mod shap;

pub use self::progress::*;

use ndarray::prelude::*;
use thiserror::Error;

#[derive(Debug)]
pub struct TrainOptions {
	/// The percent of the dataset to set aside for computing early stopping metrics.
	pub early_stopping_fraction: f32,
	/// l2_regularization
	pub l2_regularization: f32,
	/// learning_rate
	pub learning_rate: f32,
	/// The maximum number of epochs to train.
	pub max_epochs: usize,
	/// The number of examples to use for each batch of training.
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

#[derive(Debug, Error)]
pub enum Error {
	#[error("invalid label column")]
	InvalidLabelColumn,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Model {
	Regressor(Regressor),
	BinaryClassifier(BinaryClassifier),
	MulticlassClassifier(MulticlassClassifier),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Regressor {
	pub bias: f32,
	pub weights: Array1<f32>,
	pub means: Vec<f32>,
	pub losses: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryClassifier {
	pub weights: Array1<f32>,
	pub bias: f32,
	pub means: Vec<f32>,
	pub losses: Vec<f32>,
	pub classes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MulticlassClassifier {
	pub weights: Array2<f32>,
	pub biases: Array1<f32>,
	pub means: Vec<f32>,
	pub losses: Vec<f32>,
	pub classes: Vec<String>,
}
