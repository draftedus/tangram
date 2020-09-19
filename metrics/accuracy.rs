use super::{mean::Mean, StreamingMetric};

/// The accuracy is the proportion of examples where predicted == label.
#[derive(Default)]
pub struct Accuracy(Mean);

impl Accuracy {
	pub fn new() -> Self {
		Self::default()
	}
}

impl StreamingMetric<'_> for Accuracy {
	type Input = (usize, usize);
	type Output = Option<f32>;

	fn update(&mut self, value: Self::Input) {
		self.0.update(if value.0 == value.1 { 1.0 } else { 0.0 })
	}

	fn merge(&mut self, other: Self) {
		self.0.merge(other.0)
	}

	fn finalize(self) -> Option<f32> {
		self.0.finalize()
	}
}
