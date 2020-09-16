use super::{mean::Mean, StreamingMetric};

/// The mean squared error is the sum of squared differences between the predicted value and the label.
#[derive(Default)]
pub struct MeanSquaredError(Mean);

impl StreamingMetric<'_> for MeanSquaredError {
	type Input = (f32, f32);
	type Output = Option<f32>;

	fn update(&mut self, value: Self::Input) {
		self.0.update((value.1 - value.0).powi(2))
	}

	fn merge(&mut self, other: Self) {
		self.0.merge(other.0)
	}

	fn finalize(self) -> Self::Output {
		self.0.finalize()
	}
}
