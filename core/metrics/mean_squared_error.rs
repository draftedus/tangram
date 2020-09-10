use super::mean::Mean;
use super::Metric;
use ndarray::prelude::*;

#[derive(Default)]
pub struct MeanSquaredError(Mean);

impl Metric<'_> for MeanSquaredError {
	type Input = (f32, f32);
	type Output = Option<f32>;

	fn update(&mut self, value: (f32, f32)) {
		self.0.update((value.1 - value.0).powi(2))
	}

	fn merge(&mut self, other: Self) {
		self.0.merge(other.0)
	}

	fn finalize(self) -> Option<f32> {
		self.0.finalize()
	}
}

/// compute the mean squared error given predictions and labels
/// where predictions have shape (n_examples)
/// and labels have shape (n_examples)
pub fn mean_squared_error(predictions: ArrayView1<f32>, labels: ArrayView1<f32>) -> f32 {
	let mut metric = MeanSquaredError::default();
	predictions
		.iter()
		.zip(labels.iter())
		.for_each(|(p, l)| metric.update((*p, *l)));
	metric.finalize().unwrap()
}
