use super::{shap, single, *};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::{clamp, ToPrimitive};
use std::ops::Neg;
use tangram_dataframe::*;

impl BinaryClassifier {
	/// Train a Tree Binary Classifier.
	pub fn train(
		features: DataFrameView,
		labels: EnumColumnView,
		options: TrainOptions,
		update_progress: &mut dyn FnMut(super::Progress),
	) -> Self {
		let task = Task::BinaryClassification;
		let model = super::train::train(
			&task,
			features,
			ColumnView::Enum(labels),
			options,
			update_progress,
		);
		match model {
			Model::BinaryClassifier(model) => model,
			_ => unreachable!(),
		}
	}

	/// Make predictions on a Tree Binary Classifier.
	pub fn predict(&self, features: ArrayView2<Value>, mut probabilities: ArrayViewMut2<f32>) {
		let mut logits = probabilities.column_mut(1);
		logits.fill(self.bias);
		for (example_index, logit) in logits.iter_mut().enumerate() {
			for tree in &self.trees {
				let mut row = vec![Value::Number(0.0); features.ncols()];
				row.iter_mut()
					.zip(features.row(example_index))
					.for_each(|(v, feature)| {
						*v = *feature;
					});
				*logit += tree.predict(&row);
			}
		}
		for logit in logits {
			*logit = 1.0 / (logit.neg().exp() + 1.0);
		}
		let (mut probabilities_neg, probabilities_pos) = probabilities.split_at(Axis(1), 1);
		izip!(probabilities_neg.view_mut(), probabilities_pos.view())
			.for_each(|(neg, pos)| *neg = 1.0 - *pos);
	}

	/// Compute SHAP values.
	pub fn compute_shap_values(
		&self,
		features: ArrayView2<Value>,
		mut shap_values: ArrayViewMut3<f32>,
	) {
		let trees = ArrayView1::from_shape(self.trees.len(), &self.trees).unwrap();
		izip!(
			features.axis_iter(Axis(0)),
			shap_values.axis_iter_mut(Axis(0)),
		)
		.for_each(|(features, mut shap_values)| {
			let mut row = vec![Value::Number(0.0); features.len()];
			row.iter_mut().zip(features).for_each(|(v, feature)| {
				*v = *feature;
			});
			let x = shap::compute_shap(row.as_slice(), trees, self.bias);
			shap_values.row_mut(0).assign(&Array1::from(x));
		});
	}
}

/// Update the logits with the predictions from a single round of trees.
pub fn update_logits(
	trees: &[single::TrainTree],
	features: ArrayView2<u8>,
	mut logits: ArrayViewMut2<f32>,
) {
	for tree in trees {
		for (logit, features) in logits.iter_mut().zip(features.genrows()) {
			*logit += tree.predict(features);
		}
	}
}

/// Compute the binary cross entropy loss.
pub fn compute_loss(labels: ArrayView1<usize>, logits: ArrayView2<f32>) -> f32 {
	let mut total = 0.0;
	for (label, logit) in labels.iter().zip(logits) {
		let label = (*label - 1).to_f32().unwrap();
		let probability = 1.0 / (logit.neg().exp() + 1.0);
		let probability_clamped = clamp(probability, std::f32::EPSILON, 1.0 - std::f32::EPSILON);
		total += -1.0 * label * probability_clamped.ln()
			+ -1.0 * (1.0 - label) * (1.0 - probability_clamped).ln()
	}
	total / labels.len().to_f32().unwrap()
}

/// Compute the biases.
pub fn compute_biases(labels: ArrayView1<usize>) -> Array1<f32> {
	// positive label = 2
	// negative label = 1
	let pos_count = labels.sum() - labels.len();
	let neg_count = labels.len() - pos_count;
	let log_odds = (pos_count.to_f32().unwrap() / neg_count.to_f32().unwrap()).ln();
	arr1(&[log_odds])
}

/// Compute the gradients and hessians for each example given the labels and predictions.
pub fn update_gradients_and_hessians(
	// (n_trees_per_round, n_examples)
	mut gradients: ArrayViewMut2<f32>,
	// (n_trees_per_round, n_examples)
	mut hessians: ArrayViewMut2<f32>,
	// (n_examples)
	labels: ArrayView1<usize>,
	// (n_trees_per_rounds, n_examples)
	predictions: ArrayView2<f32>,
) {
	izip!(
		gradients.row_mut(0),
		hessians.row_mut(0),
		labels,
		predictions.row(0)
	)
	.for_each(|(gradient, hessian, label, prediction)| {
		let probability = clamp(
			sigmoid(*prediction),
			std::f32::EPSILON,
			1.0 - std::f32::EPSILON,
		);
		*gradient = probability - (label - 1).to_f32().unwrap();
		*hessian = probability * (1.0 - probability);
	});
}

fn sigmoid(value: f32) -> f32 {
	1.0 / (value.neg().exp() + 1.0)
}
