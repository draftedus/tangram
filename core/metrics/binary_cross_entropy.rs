use super::mean::Mean;
use super::Metric;
use num_traits::clamp;

#[derive(Debug, Default)]
pub struct BinaryCrossEntropy(Mean);

pub struct BinaryCrossEntropyInput {
	pub probability: f32,
	// 1-indexed
	pub label: usize,
}

impl Metric<'_> for BinaryCrossEntropy {
	type Input = BinaryCrossEntropyInput;
	type Output = Option<f32>;

	fn update(&mut self, value: BinaryCrossEntropyInput) {
		self.0
			.update(binary_cross_entropy(value.probability, value.label));
	}

	fn merge(&mut self, other: Self) {
		self.0.merge(other.0)
	}

	fn finalize(self) -> Option<f32> {
		self.0.finalize()
	}
}

pub fn binary_cross_entropy(probability: f32, label: usize) -> f32 {
	let label = match label {
		1 => 0.0,
		2 => 1.0,
		_ => unreachable!(),
	};
	let probability_clamped = clamp(probability, std::f32::EPSILON, 1.0 - std::f32::EPSILON);
	-1.0 * label * probability_clamped.ln()
		+ -1.0 * (1.0 - label) * (1.0 - probability_clamped).ln()
}
