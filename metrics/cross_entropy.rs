use super::{mean::Mean, StreamingMetric};
use ndarray::prelude::*;
use num_traits::clamp;

/// CrossEntropy is the loss function used in multiclass classification. [Learn more](https://en.wikipedia.org/wiki/Cross_entropy#Cross-entropy_loss_function_and_logistic_regression).
#[derive(Default)]
pub struct CrossEntropy(Mean);

/// The input to [CrossEntropy](struct.CrossEntropy.html).
pub struct CrossEntropyInput<'a> {
	/// (n_classes)
	pub probabilities: ArrayView1<'a, f32>,
	pub label: usize,
}

/// The output from [CrossEntropy](struct.CrossEntropy.html).
pub struct CrossEntropyOutput(pub Option<f32>);

impl<'a> StreamingMetric<'a> for CrossEntropy {
	type Input = CrossEntropyInput<'a>;
	type Output = CrossEntropyOutput;

	fn update(&mut self, value: CrossEntropyInput) {
		let label = value.label.checked_sub(1).unwrap();
		let mut total = 0.0;
		for (index, &probability) in value.probabilities.indexed_iter() {
			if index == label {
				total += -clamp(probability, std::f32::EPSILON, 1.0 - std::f32::EPSILON).ln();
			}
		}
		self.0.update(total)
	}

	fn merge(&mut self, other: Self) {
		self.0.merge(other.0)
	}

	fn finalize(self) -> Self::Output {
		CrossEntropyOutput(self.0.finalize())
	}
}
