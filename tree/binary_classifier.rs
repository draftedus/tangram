use super::{
	shap,
	train::{Model, TrainTree},
	TrainOptions, Tree,
};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::{clamp, ToPrimitive};
use rayon::prelude::*;
use std::num::NonZeroUsize;
use std::ops::Neg;
use tangram_dataframe::*;
use tangram_thread_pool::pzip;

/// `BinaryClassifier`s predict binary target values, for example whether a patient has heart disease or not.
#[derive(Debug)]
pub struct BinaryClassifier {
	/// The initial prediction of the model given no trained trees. The bias is calculated using the distribution of the unique values in target column in the training dataset.
	pub bias: f32,
	/// The trees for this model.
	pub trees: Vec<Tree>,
	/// The importance of each feature as measured by the number of times the feature was used in a branch node.
	pub feature_importances: Option<Vec<f32>>,
	/// The training losses in each round of training this model.
	pub losses: Option<Vec<f32>>,
	/// The names of the unique values in the target column.
	pub classes: Vec<String>,
}

impl BinaryClassifier {
	/// Train a binary classifier.
	pub fn train(
		features: DataFrameView,
		labels: EnumColumnView,
		train_options: TrainOptions,
		update_progress: &mut dyn FnMut(super::TrainProgress),
	) -> Self {
		let task = crate::train::Task::BinaryClassification;
		let model = crate::train::train(
			task,
			features,
			ColumnView::Enum(labels),
			train_options,
			update_progress,
		);
		match model {
			Model::BinaryClassifier(model) => model,
			_ => unreachable!(),
		}
	}

	/// Make predictions.
	pub fn predict(&self, features: ArrayView2<Value>, mut probabilities: ArrayViewMut2<f32>) {
		let mut predictions = probabilities.column_mut(1);
		predictions.fill(self.bias);
		for (example_index, logit) in predictions.iter_mut().enumerate() {
			for tree in &self.trees {
				let mut row = vec![Value::Number(0.0); features.ncols()];
				for (v, feature) in row.iter_mut().zip(features.row(example_index)) {
					*v = *feature;
				}
				*logit += tree.predict(&row);
			}
		}
		for prediction in predictions {
			*prediction = 1.0 / (prediction.neg().exp() + 1.0);
		}
		let (mut probabilities_neg, probabilities_pos) = probabilities.split_at(Axis(1), 1);
		for (neg, pos) in izip!(probabilities_neg.view_mut(), probabilities_pos.view()) {
			*neg = 1.0 - *pos
		}
	}

	/// Compute SHAP values.
	pub fn compute_shap_values(
		&self,
		features: ArrayView2<Value>,
		mut shap_values: ArrayViewMut3<f32>,
	) {
		let trees = ArrayView1::from_shape(self.trees.len(), &self.trees).unwrap();
		for (features, mut shap_values) in izip!(
			features.axis_iter(Axis(0)),
			shap_values.axis_iter_mut(Axis(0)),
		) {
			let mut row = vec![Value::Number(0.0); features.len()];
			for (v, feature) in row.iter_mut().zip(features) {
				*v = *feature;
			}
			shap::compute_shap(
				row.as_slice(),
				trees,
				self.bias,
				shap_values.row_mut(0).as_slice_mut().unwrap(),
			);
		}
	}
}

/// This function is used by the common train function to update the logits after each tree is trained for binary classification.
pub fn update_logits(
	trees_for_round: &[TrainTree],
	binned_features: ArrayView2<Value>,
	mut predictions: ArrayViewMut2<f32>,
) {
	for tree in trees_for_round {
		for (prediction, features) in predictions
			.iter_mut()
			.zip(binned_features.axis_iter(Axis(0)))
		{
			*prediction += tree.predict(features.as_slice().unwrap());
		}
	}
}

/// This function is used by the common train function to compute the loss after each tree is trained for binary classification.
pub fn compute_loss(labels: ArrayView1<Option<NonZeroUsize>>, logits: ArrayView2<f32>) -> f32 {
	let mut total = 0.0;
	for (label, logit) in labels.iter().zip(logits) {
		let label = (label.unwrap().get() - 1).to_f32().unwrap();
		let probability = 1.0 / (logit.neg().exp() + 1.0);
		let probability_clamped = clamp(probability, std::f32::EPSILON, 1.0 - std::f32::EPSILON);
		total += -1.0 * label * probability_clamped.ln()
			+ -1.0 * (1.0 - label) * (1.0 - probability_clamped).ln()
	}
	total / labels.len().to_f32().unwrap()
}

/// This function is used by the common train function to compute the biases for binary classification.
pub fn compute_biases(labels: ArrayView1<Option<NonZeroUsize>>) -> Array1<f32> {
	let pos_count = labels
		.iter()
		.map(|l| if l.unwrap().get() == 2 { 1 } else { 0 })
		.sum::<usize>();
	let neg_count = labels.len() - pos_count;
	let log_odds = (pos_count.to_f32().unwrap() / neg_count.to_f32().unwrap()).ln();
	arr1(&[log_odds])
}

/// This function is used by the common train function to compute the gradients and hessian after each round.
pub fn compute_gradients_and_hessians(
	// (n_examples)
	gradients: &mut [f32],
	// (n_examples)
	hessians: &mut [f32],
	// (n_examples)
	labels: &[Option<NonZeroUsize>],
	// (n_examples)
	predictions: &[f32],
) {
	pzip!(gradients, hessians, labels, predictions).for_each(
		|(gradient, hessian, label, prediction)| {
			let probability = clamp(
				sigmoid(*prediction),
				std::f32::EPSILON,
				1.0 - std::f32::EPSILON,
			);
			*gradient = probability - (label.unwrap().get() - 1).to_f32().unwrap();
			*hessian = probability * (1.0 - probability);
		},
	);
}

fn sigmoid(value: f32) -> f32 {
	1.0 / (value.neg().exp() + 1.0)
}
