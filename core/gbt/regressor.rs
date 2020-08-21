use super::{shap, tree, types};
use crate::dataframe::*;
use ndarray::prelude::*;
use ndarray::Zip;
use num_traits::ToPrimitive;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

impl types::Regressor {
	pub fn train(
		features: DataFrameView,
		labels: NumberColumnView,
		options: types::TrainOptions,
		update_progress: &mut dyn FnMut(super::Progress),
	) -> Self {
		let task = types::Task::Regression;
		let model = super::train::train(
			&task,
			features,
			ColumnView::Number(labels.clone()),
			options,
			update_progress,
		);
		match model {
			types::Model::Regressor(model) => model,
			_ => unreachable!(),
		}
	}

	pub fn predict(
		&self,
		features: ArrayView2<Value>,
		mut predictions: ArrayViewMut1<f32>,
		mut shap_values: Option<ArrayViewMut3<f32>>,
	) {
		predictions.fill(self.bias);
		let mut row = vec![Value::Number(0.0); features.ncols()];
		for (i, prediction) in predictions.iter_mut().enumerate() {
			for tree in &self.trees {
				row.iter_mut()
					.zip(features.row(i))
					.for_each(|(v, feature)| {
						*v = *feature;
					});
				*prediction += tree.predict(&row);
			}
		}

		let trees = ArrayView1::from_shape(self.trees.len(), &self.trees).unwrap();
		if let Some(shap_values) = &mut shap_values {
			(
				features.axis_iter(Axis(0)),
				shap_values.axis_iter_mut(Axis(0)),
			)
				.into_par_iter()
				.for_each(|(features, mut shap_values)| {
					// n_examples times
					let mut row = vec![Value::Number(0.0); features.len()];
					row.iter_mut().zip(features).for_each(|(v, feature)| {
						*v = *feature;
					});
					let x = shap::compute_shap(row.as_slice(), trees, self.bias);
					shap_values.row_mut(0).assign(&x);
				});
		}
	}
}

pub fn update_logits(
	trees: &[tree::types::TrainTree],
	features: ArrayView2<u8>,
	mut predictions: ArrayViewMut2<f32>,
) {
	Zip::from(predictions.row_mut(0))
		.and(features.genrows())
		.apply(|prediction, features| {
			for tree in trees {
				*prediction += tree.predict(features);
			}
		});
}

/// squared error loss
/// loss = 0.5 * (label - prediction)^2
pub fn compute_loss(labels: ArrayView1<f32>, predictions: ArrayView2<f32>) -> f32 {
	let mut loss = 0.0;
	for (label, prediction) in labels.iter().zip(predictions) {
		loss += 0.5 * (label - prediction).powi(2);
	}
	loss / labels.len().to_f32().unwrap()
}

pub fn compute_biases(labels: ArrayView1<f32>) -> Array1<f32> {
	arr1(&[labels.mean().unwrap()])
}

pub fn update_gradients_and_hessians(
	// (n_trees_per_round, n_examples)
	mut gradients: ArrayViewMut2<f32>,
	// (n_trees_per_round, n_examples)
	_hessians: ArrayViewMut2<f32>,
	// (n_examples)
	labels: ArrayView1<f32>,
	// (n_trees_per_round, n_examples)
	predictions: ArrayView2<f32>,
) {
	// gradients are y_pred - y_true
	// d / dy_pred (0.5 ( y_pred - y_true) **2 ) = 2 * (0.5) * (y_pred - y_pred) = y_pred - y_true
	Zip::from(gradients.row_mut(0))
		.and(labels)
		.and(predictions.row(0))
		.par_apply(|gradient, label, prediction| *gradient = prediction - label)
}
