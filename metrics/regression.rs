use super::mean_variance::{MeanVariance, MeanVarianceOutput};
use super::StreamingMetric;
use itertools::izip;
use num_traits::ToPrimitive;

/// RegressionMetrics computes metrics used to evaluate regressors.
pub struct RegressionMetrics {
	mean_variance: MeanVariance,
	absolute_error: f64,
	squared_error: f64,
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
}

impl RegressionMetrics {
	pub fn new() -> RegressionMetrics {
		RegressionMetrics::default()
	}
}

impl Default for RegressionMetrics {
	fn default() -> RegressionMetrics {
		RegressionMetrics {
			mean_variance: MeanVariance::default(),
			absolute_error: 0.0,
			squared_error: 0.0,
		}
	}
}

impl<'a> StreamingMetric<'a> for RegressionMetrics {
	type Input = RegressionMetricsInput<'a>;
	type Output = RegressionMetricsOutput;

	fn update(&mut self, input: RegressionMetricsInput) {
		for (prediction, label) in izip!(input.predictions.iter(), input.labels.iter()) {
			self.mean_variance.update(*label);
			let absolute_error = prediction - label;
			let squared_error = absolute_error * absolute_error;
			self.absolute_error += absolute_error as f64;
			self.squared_error += squared_error as f64;
		}
	}

	fn merge(&mut self, other: Self) {
		self.mean_variance.merge(other.mean_variance);
		self.absolute_error += other.absolute_error;
		self.squared_error += other.squared_error;
	}

	fn finalize(self) -> Self::Output {
		let MeanVarianceOutput { variance, n, .. } = self.mean_variance.finalize();
		let mae = self.absolute_error / n.to_f64().unwrap();
		let mse = self.squared_error / n.to_f64().unwrap();
		let rmse = mse.sqrt();
		let r2 = 1.0 - self.squared_error / (variance as f64 * n.to_f64().unwrap());
		RegressionMetricsOutput {
			mae: mae.to_f32().unwrap(),
			mse: mse.to_f32().unwrap(),
			r2: r2.to_f32().unwrap(),
			rmse: rmse.to_f32().unwrap(),
		}
	}
}
