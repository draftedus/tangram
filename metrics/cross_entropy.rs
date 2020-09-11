use super::{mean::Mean, Metric};
use ndarray::prelude::*;
use num_traits::clamp;

/// CrossEntropy is the loss function used in multiclass classifiers. See [Cross Entropy](https://en.wikipedia.org/wiki/Cross_entropy#Cross-entropy_loss_function_and_logistic_regression).
#[derive(Default)]
pub struct CrossEntropy(Mean);

pub struct CrossEntropyInput<'a> {
	/// (n_classes)
	pub probabilities: ArrayView1<'a, f32>,
	pub label: usize,
}

pub type CrossEntropyOutput = Option<f32>;

impl<'a> Metric<'a> for CrossEntropy {
	type Input = CrossEntropyInput<'a>;
	type Output = CrossEntropyOutput;

	fn update(&mut self, value: CrossEntropyInput) {
		self.0
			.update(cross_entropy(value.probabilities, value.label))
	}

	fn merge(&mut self, other: Self) {
		self.0.merge(other.0)
	}

	fn finalize(self) -> Option<f32> {
		self.0.finalize()
	}
}

/// This function computes the cross_entropy given a probability array of size (n_classes,) and a class label that is 1-indexed.
pub fn cross_entropy(probabilities: ArrayView1<f32>, label: usize) -> f32 {
	// labels are 1-indexed, convert to 0-indexed
	let label = label.checked_sub(1).unwrap();
	let mut total = 0.0;
	for (index, &probability) in probabilities.indexed_iter() {
		let probability = clamp(probability, std::f32::EPSILON, 1.0 - std::f32::EPSILON);
		if index == label {
			total += -probability.ln();
		}
	}
	total
}
