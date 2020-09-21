use super::{
	shap,
	train::{Model, TrainTree},
	TrainOptions, Tree,
};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use tangram_dataframe::*;

/// `Regressor`s predict continuous target values, for example the selling price of a home.
#[derive(Debug)]
pub struct Regressor {
	/// The initial prediction of the model given no trained trees. The bias is calculated using the mean value of the target column in the training dataset.
	pub bias: f32,
	/// The trees for this model.
	pub trees: Vec<Tree>,
	/// The importance of each feature as measured by the number of times the feature was used in a branch node.
	pub feature_importances: Option<Vec<f32>>,
	/// The training losses in each round of training this model.
	pub losses: Option<Vec<f32>>,
}

impl Regressor {
	/// Train a regressor.
	pub fn train(
		features: DataFrameView,
		labels: NumberColumnView,
		options: TrainOptions,
		update_progress: &mut dyn FnMut(super::TrainProgress),
	) -> Self {
		let task = crate::train::Task::Regression;
		let model = crate::train::train(
			&task,
			features,
			ColumnView::Number(labels),
			options,
			update_progress,
		);
		match model {
			Model::Regressor(model) => model,
			_ => unreachable!(),
		}
	}

	/// Make predictions.
	pub fn predict(&self, features: ArrayView2<Value>, mut predictions: ArrayViewMut1<f32>) {
		predictions.fill(self.bias);
		let mut row = vec![Value::Number(0.0); features.ncols()];
		for (i, prediction) in predictions.iter_mut().enumerate() {
			for tree in &self.trees {
				for (v, feature) in row.iter_mut().zip(features.row(i)) {
					*v = *feature;
				}
				*prediction += tree.predict(&row);
			}
		}
	}

	/// Compute SHAP values.
	pub fn compute_shap_values(
		&self,
		features: ArrayView2<Value>,
		// TODO change this type?
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

/// This function is used by the common train function to update the logits after each round of trees is trained for regression.
pub fn update_logits(
	trees: &[TrainTree],
	features: ArrayView2<Value>,
	mut predictions: ArrayViewMut2<f32>,
) {
	for (prediction, features) in izip!(predictions.row_mut(0), features.genrows()) {
		for tree in trees {
			*prediction += tree.predict(features.as_slice().unwrap());
		}
	}
}

/// This function is used by the common train function to compute the loss after each tree is trained for regression.
pub fn compute_loss(labels: ArrayView1<f32>, predictions: ArrayView2<f32>) -> f32 {
	let mut loss = 0.0;
	for (label, prediction) in labels.iter().zip(predictions) {
		loss += 0.5 * (label - prediction).powi(2);
	}
	loss / labels.len().to_f32().unwrap()
}

/// This function is used by the common train function to compute the biases for regression.
pub fn compute_biases(labels: ArrayView1<f32>) -> Array1<f32> {
	arr1(&[labels.mean().unwrap()])
}

/// This function is used by the common train function to compute the gradients and hessian after each round.
pub fn compute_gradients_and_hessians(
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
	for (gradient, label, prediction) in izip!(gradients.row_mut(0), labels, predictions.row(0)) {
		*gradient = prediction - label;
	}
}
