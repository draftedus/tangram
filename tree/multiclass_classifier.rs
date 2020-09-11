use super::{shap, single, Model, MulticlassClassifier, Task, TrainOptions};
use crate::dataframe::*;
use itertools::izip;
use ndarray::prelude::*;
use num_traits::{clamp, ToPrimitive};

impl MulticlassClassifier {
	pub fn train(
		features: DataFrameView,
		labels: EnumColumnView,
		options: TrainOptions,
		update_progress: &mut dyn FnMut(super::Progress),
	) -> Self {
		let task = Task::MulticlassClassification {
			n_trees_per_round: labels.options.len(),
		};
		let model = super::train::train(
			&task,
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

	pub fn predict(
		&self,
		features: ArrayView2<Value>,
		probabilities: ArrayViewMut2<f32>,
		mut shap_values: Option<ArrayViewMut3<f32>>,
		// progress: &dyn Fn(),
	) {
		let n_rounds = self.n_rounds;
		let n_classes = self.n_classes;
		let trees = ArrayView2::from_shape((n_rounds, n_classes), &self.trees).unwrap();
		let mut logits = probabilities;
		let biases = ArrayView1::from_shape(n_classes, &self.biases).unwrap();
		izip!(logits.axis_iter_mut(Axis(0)), features.axis_iter(Axis(0))).for_each(
			|(mut logits, features)| {
				let mut row = vec![Value::Number(0.0); features.len()];
				row.iter_mut().zip(features).for_each(|(v, feature)| {
					*v = *feature;
				});
				logits.assign(&biases);
				for trees in trees.genrows() {
					for (logit, tree) in logits.iter_mut().zip(trees.iter()) {
						*logit += tree.predict(&row);
					}
				}
				softmax(logits);
			},
		);
		if let Some(shap_values) = &mut shap_values {
			izip!(
				features.axis_iter(Axis(0)),
				shap_values.axis_iter_mut(Axis(0)),
			)
			.for_each(|(features, mut shap_values)| {
				let mut row = vec![Value::Number(0.0); features.len()];
				row.iter_mut().zip(features).for_each(|(v, feature)| {
					*v = *feature;
				});
				for class_index in 0..n_classes {
					let x = shap::compute_shap(
						row.as_slice(),
						trees.column(class_index),
						biases[class_index],
					);
					shap_values.row_mut(class_index).assign(&Array1::from(x));
				}
			});
		}
	}
}

// updates the logits with a single round of trees
pub fn update_logits(
	trees: &[single::TrainTree],
	features: ArrayView2<u8>,
	mut logits: ArrayViewMut2<f32>,
) {
	let features_rows = features.genrows().into_iter();
	let logits_rows = logits.gencolumns_mut().into_iter();
	for (features, mut logits) in features_rows.zip(logits_rows) {
		for (logit, tree) in logits.iter_mut().zip(trees.iter()) {
			*logit += tree.predict(features);
		}
	}
}

pub fn compute_loss(labels: ArrayView1<usize>, logits: ArrayView2<f32>) -> f32 {
	let mut loss = 0.0;
	for (label, logits) in labels.into_iter().zip(logits.gencolumns()) {
		let mut probabilities = logits.to_owned();
		softmax(probabilities.view_mut());
		for (index, &probability) in probabilities.indexed_iter() {
			let probability = clamp(probability, std::f32::EPSILON, 1.0 - std::f32::EPSILON);
			if index == (*label - 1) {
				loss += -probability.ln();
			}
		}
	}
	loss / labels.len().to_f32().unwrap()
}

pub fn compute_biases(labels: ArrayView1<usize>, n_trees_per_round: usize) -> Array1<f32> {
	let mut baseline: Array1<f32> = Array::zeros(n_trees_per_round);
	for label in labels {
		let label = *label - 1;
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

pub fn update_gradients_and_hessians(
	// (n_trees_per_round, n_examples)
	mut gradients: ArrayViewMut2<f32>,
	// (n_trees_per_round, n_examples)
	mut hessians: ArrayViewMut2<f32>,
	// (n_examples)
	labels: ArrayView1<usize>,
	// (n_trees_per_round, n_examples)
	predictions: ArrayView2<f32>,
) {
	let mut predictions = predictions.to_owned();
	izip!(
		gradients.gencolumns_mut(),
		hessians.gencolumns_mut(),
		predictions.gencolumns_mut(),
		labels
	)
	.for_each(|(mut gradients, mut hessians, mut predictions, label)| {
		softmax(predictions.view_mut());
		izip!(
			predictions.iter().enumerate(),
			gradients.iter_mut(),
			hessians.iter_mut()
		)
		.for_each(|((class_index, prediction), gradient, hessian)| {
			let label = if (label - 1) == class_index { 1.0 } else { 0.0 };
			*gradient = *prediction - label;
			*hessian = *prediction * (1.0 - *prediction);
		});
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
