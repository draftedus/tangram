use super::StreamingMetric;
use num_traits::ToPrimitive;
use std::num::NonZeroU64;

/// RegressionMetrics computes metrics used to evaluate regressors.
pub struct RegressionMetrics {
	mean_variance: Option<MeanVariance>,
	absolute_error: f64,
	squared_error: f64,
}

/// MeanVariance holds the information needed to compute streaming mean and variance. It is required that `n` be >= 1, so you should use `Option<MeanVariance>` if `n` may be zero.
struct MeanVariance {
	pub n: NonZeroU64,
	pub m2: f64,
	pub mean: f64,
}

/// The input to [RegressionMetrics](struct.RegressionMetrics.html).
pub struct RegressionMetricsInput<'a> {
	pub predictions: &'a [f32],
	pub labels: &'a [f32],
}

/// The output from [RegressionMetrics](struct.RegressionMetrics.html).
#[derive(Debug)]
pub struct RegressionMetricsOutput {
	/// The mean squared error is equal to the mean of the squared errors. For a given example, the error is the difference between the true value and the model's predicted value.
	pub mse: f32,
	/// The root mean squared error is equal to the square root of the mean squared error.
	pub rmse: f32,
	/// The mean of the absolute value of the errors.
	pub mae: f32,
	/// The r-squared value. https://en.wikipedia.org/wiki/Coefficient_of_determination.
	pub r2: f32,
	/// The baseline mean squared error is the mean squared error if the model always predicted the mean value.
	pub baseline_mse: f32,
	/// The baseline root mean squared error is the square root of the baseline mean squared error.
	pub baseline_rmse: f32,
}

impl RegressionMetrics {
	pub fn new() -> RegressionMetrics {
		RegressionMetrics::default()
	}
}

impl Default for RegressionMetrics {
	fn default() -> RegressionMetrics {
		RegressionMetrics {
			mean_variance: None,
			absolute_error: 0.0,
			squared_error: 0.0,
		}
	}
}

impl<'a> StreamingMetric<'a> for RegressionMetrics {
	type Input = RegressionMetricsInput<'a>;
	type Output = RegressionMetricsOutput;

	fn update(&mut self, input: RegressionMetricsInput) {
		let RegressionMetricsInput {
			predictions,
			labels,
		} = input;
		for (prediction, label) in predictions.iter().zip(labels.iter()) {
			match &mut self.mean_variance {
				Some(mean_variance) => {
					let (mean, m2) = merge_mean_m2(
						mean_variance.n.get(),
						mean_variance.mean,
						mean_variance.m2,
						1,
						*label as f64,
						0.0,
					);
					mean_variance.n = NonZeroU64::new(mean_variance.n.get() + 1).unwrap();
					mean_variance.mean = mean;
					mean_variance.m2 = m2;
				}
				None => {
					self.mean_variance = Some(MeanVariance {
						n: NonZeroU64::new(1).unwrap(),
						mean: *label as f64,
						m2: 0.0,
					})
				}
			}
			let absolute_error = prediction - label;
			let squared_error = absolute_error * absolute_error;
			self.absolute_error += absolute_error as f64;
			self.squared_error += squared_error as f64;
		}
	}

	fn merge(&mut self, other: Self) {
		match &mut self.mean_variance {
			Some(mean_variance) => {
				if let Some(other) = other.mean_variance {
					let (mean, m2) = merge_mean_m2(
						mean_variance.n.get(),
						mean_variance.mean,
						mean_variance.m2,
						other.n.get(),
						other.mean,
						other.m2,
					);
					mean_variance.mean = mean;
					mean_variance.m2 = m2;
					mean_variance.n =
						NonZeroU64::new(mean_variance.n.get() + other.n.get()).unwrap();
				}
			}
			None => {
				self.mean_variance = other.mean_variance;
			}
		}
		self.absolute_error += other.absolute_error;
		self.squared_error += other.squared_error;
	}

	fn finalize(self) -> Self::Output {
		let (n, variance) = match self.mean_variance {
			Some(m) => (
				m.n.get().to_f64().unwrap(),
				m.m2 / m.n.get().to_f64().unwrap(),
			),
			None => (0.0, f64::NAN),
		};
		let mae = self.absolute_error / n;
		let mse = self.squared_error / n;
		let rmse = mse.sqrt();
		let r2 = 1.0 - self.squared_error / (variance * n);
		let baseline_mse = variance;
		let baseline_rmse = baseline_mse.sqrt();
		RegressionMetricsOutput {
			mae: mae.to_f32().unwrap(),
			mse: mse.to_f32().unwrap(),
			r2: r2.to_f32().unwrap(),
			rmse: rmse.to_f32().unwrap(),
			baseline_mse: baseline_mse.to_f32().unwrap(),
			baseline_rmse: baseline_rmse.to_f32().unwrap(),
		}
	}
}

/// This function combines two separate means and variances into a single mean and variance which is useful in parallel algorithms.
pub fn merge_mean_m2(
	n_a: u64,
	mean_a: f64,
	m2_a: f64,
	n_b: u64,
	mean_b: f64,
	m2_b: f64,
) -> (f64, f64) {
	let n_a = n_a.to_f64().unwrap();
	let n_b = n_b.to_f64().unwrap();
	(
		(((n_a * mean_a) + (n_b * mean_b)) / (n_a + n_b)),
		m2_a + m2_b + (mean_b - mean_a) * (mean_b - mean_a) * (n_a * n_b / (n_a + n_b)),
	)
}

/// This function computes the variance given the `m2` and `n`.
pub fn m2_to_variance(m2: f64, n: NonZeroU64) -> f32 {
	(m2 / n.get().to_f64().unwrap()).to_f32().unwrap()
}
