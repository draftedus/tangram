use super::{
	shap,
	train::{Model, TrainTree},
	TrainOptions, Tree,
};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::{clamp, ToPrimitive};
use std::num::NonZeroUsize;
use tangram_dataframe::*;

/// `MulticlasClassifier`s predict multiclass target values, for example which of several species a flower is.
#[derive(Debug)]
pub struct MulticlassClassifier {
	/// The initial prediction of the model given no trained trees. The bias is calculated using the distribution of the unique values in target column in the training dataset.
	pub biases: Vec<f32>,
	/// The trees for this model. It has shape (n_rounds, n_classes) because for each round, we train n_classes trees.
	pub trees: Vec<Tree>,
	/// The number of classes.
	pub n_classes: usize,
	/// The number of rounds.
	pub n_rounds: usize,
	/// The importance of each feature as measured by the number of times the feature was used in a branch node.
	pub feature_importances: Option<Vec<f32>>,
	/// The training losses in each round of training this model.
	pub losses: Option<Vec<f32>>,
	/// The names of the unique values in the target column.
	pub classes: Vec<String>,
}

impl MulticlassClassifier {
	// Train a multiclass classifier.
	pub fn train(
		features: DataFrameView,
		labels: EnumColumnView,
		options: TrainOptions,
		update_progress: &mut dyn FnMut(super::TrainProgress),
	) -> Self {
		let task = crate::train::Task::MulticlassClassification {
			n_classes: labels.options.len(),
		};
		let model = crate::train::train(
			task,
			features,
			ColumnView::Enum(labels),
			options,
			update_progress,
		);
		match model {
			Model::MulticlassClassifier(model) => model,
			_ => unreachable!(),
		}
	}

	// Make predictions.
	pub fn predict(&self, features: ArrayView2<Value>, mut probabilities: ArrayViewMut2<f32>) {
		let n_rounds = self.n_rounds;
		let n_classes = self.n_classes;
		let trees = ArrayView2::from_shape((n_rounds, n_classes), &self.trees).unwrap();
		let biases = ArrayView1::from_shape(n_classes, &self.biases).unwrap();
		for (mut logits, features) in izip!(
			probabilities.axis_iter_mut(Axis(0)),
			features.axis_iter(Axis(0))
		) {
			let mut row = vec![Value::Number(0.0); features.len()];
			for (v, feature) in row.iter_mut().zip(features) {
				*v = *feature;
			}
			logits.assign(&biases);
			for trees in trees.genrows() {
				for (logit, tree) in logits.iter_mut().zip(trees.iter()) {
					*logit += tree.predict(&row);
				}
			}
			softmax(logits);
		}
	}

	/// Compute SHAP values.
	pub fn compute_shap_values(
		&self,
		features: ArrayView2<Value>,
		mut shap_values: ArrayViewMut3<f32>,
	) {
		let n_rounds = self.n_rounds;
		let n_classes = self.n_classes;
		let trees = ArrayView2::from_shape((n_rounds, n_classes), &self.trees).unwrap();
		let biases = ArrayView1::from_shape(n_classes, &self.biases).unwrap();
		for (features, mut shap_values) in izip!(
			features.axis_iter(Axis(0)),
			shap_values.axis_iter_mut(Axis(0)),
		) {
			let mut row = vec![Value::Number(0.0); features.len()];
			for (v, feature) in row.iter_mut().zip(features) {
				*v = *feature;
			}
			for class_index in 0..n_classes {
				shap::compute_shap(
					row.as_slice(),
					trees.column(class_index),
					*biases.get(class_index).unwrap(),
					shap_values.row_mut(class_index).as_slice_mut().unwrap(),
				);
			}
		}
	}
}

/// This function is used by the common train function to update the logits after each round of trees is trained for multiclass classification.
pub fn update_logits(
	trees_for_round: &[TrainTree],
	binned_features: ArrayView2<Value>,
	mut predictions: ArrayViewMut2<f32>,
) {
	let features_rows = binned_features.genrows().into_iter();
	let logits_rows = predictions.gencolumns_mut().into_iter();
	for (features, mut logits) in features_rows.zip(logits_rows) {
		for (logit, tree) in logits.iter_mut().zip(trees_for_round.iter()) {
			*logit += tree.predict(features.as_slice().unwrap());
		}
	}
}

/// This function is used by the common train function to compute the loss after each tree is trained for multiclass classification.
pub fn compute_loss(labels: ArrayView1<Option<NonZeroUsize>>, logits: ArrayView2<f32>) -> f32 {
	let mut loss = 0.0;
	for (label, logits) in labels.into_iter().zip(logits.genrows()) {
		let mut probabilities = logits.to_owned();
		softmax(probabilities.view_mut());
		for (index, &probability) in probabilities.indexed_iter() {
			let probability = clamp(probability, std::f32::EPSILON, 1.0 - std::f32::EPSILON);
			if index == (label.unwrap().get() - 1) {
				loss += -probability.ln();
			}
		}
	}
	loss / labels.len().to_f32().unwrap()
}

/// This function is used by the common train function to compute the biases for multiclass classification.
pub fn compute_biases(
	labels: ArrayView1<Option<NonZeroUsize>>,
	n_trees_per_round: usize,
) -> Array1<f32> {
	let mut baseline: Array1<f32> = Array::zeros(n_trees_per_round);
	for label in labels {
		let label = label.unwrap().get() - 1;
		baseline[label] += 1.0;
	}
	let n_examples = labels.len().to_f32().unwrap();
	baseline.mapv_inplace(|b| {
		let proba = b / n_examples;
		let clamped_proba = clamp(proba, std::f32::EPSILON, 1.0 - std::f32::EPSILON);
		clamped_proba.ln()
	});
	baseline
}

/// This function is used by the common train function to compute the gradients and hessian after each round.
pub fn compute_gradients_and_hessians(
	class_index: usize,
	// (n_examples)
	mut gradients: ArrayViewMut1<f32>,
	// (n_examples)
	mut hessians: ArrayViewMut1<f32>,
	// (n_examples)
	labels: &[Option<NonZeroUsize>],
	// (n_trees_per_round, n_examples)
	logits: ArrayView2<f32>,
) {
	izip!(
		gradients.iter_mut(),
		hessians.iter_mut(),
		logits.genrows(),
		labels.iter(),
	)
	.for_each(|(gradient, hessian, logits, label)| {
		let max = logits.iter().fold(std::f32::MIN, |a, &b| a.max(b));
		let mut sum = 0.0;
		for logit in logits.iter() {
			sum += (*logit - max).exp();
		}
		let prediction = (logits[class_index] - max).exp() / sum;
		let label = label.unwrap().get() - 1;
		let label = if label == class_index { 1.0 } else { 0.0 };
		*gradient = prediction - label;
		*hessian = prediction * (1.0 - prediction);
	});
}

fn softmax(mut logits: ArrayViewMut1<f32>) {
	let max = logits.iter().fold(std::f32::MIN, |a, &b| a.max(b));
	for logit in logits.iter_mut() {
		*logit = (*logit - max).exp();
	}
	let sum = logits.iter().sum::<f32>();
	logits /= sum;
}
