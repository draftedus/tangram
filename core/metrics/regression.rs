use super::{mean_variance::merge_mean_m2, Metric};
use ndarray::prelude::*;
use num_traits::ToPrimitive;

pub struct RegressionMetrics {
	mean_variance: Option<MeanVariance>,
	absolute_error: f64,
	squared_error: f64,
}

#[derive(Debug)]
struct MeanVariance {
	pub n: u64,
	pub m2: f64,
	pub mean: f64,
}

pub struct RegressionMetricsInput<'a> {
	pub predictions: ArrayView1<'a, f32>,
	pub labels: &'a [f32],
}

#[derive(Debug)]
pub struct RegressionMetricsOutput {
	pub mse: f32,
	pub rmse: f32,
	pub mae: f32,
	pub r2: f32,
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
}

impl Default for RegressionMetrics {
	fn default() -> Self {
		Self {
			mean_variance: None,
			absolute_error: 0.0,
			squared_error: 0.0,
		}
	}
}

impl<'a> Metric<'a> for RegressionMetrics {
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
						mean_variance.n,
						mean_variance.mean,
						mean_variance.m2,
						1,
						label.to_f64().unwrap(),
						0.0,
					);
					mean_variance.n += 1;
					mean_variance.mean = mean;
					mean_variance.m2 = m2;
				}
				None => {
					self.mean_variance = Some(MeanVariance {
						n: 1,
						mean: label.to_f64().unwrap(),
						m2: 0.0,
					})
				}
			}
			let absolute_error = prediction - label;
			let squared_error = absolute_error * absolute_error;
			self.absolute_error += absolute_error.to_f64().unwrap();
			self.squared_error += squared_error.to_f64().unwrap();
		}
	}

	fn merge(&mut self, other: Self) {
		match &mut self.mean_variance {
			Some(mean_variance) => {
				if let Some(other) = other.mean_variance {
					let (mean, m2) = merge_mean_m2(
						mean_variance.n,
						mean_variance.mean,
						mean_variance.m2,
						other.n,
						other.mean,
						other.m2,
					);
					mean_variance.mean = mean;
					mean_variance.m2 = m2;
					mean_variance.n += other.n;
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
			Some(m) => (m.n.to_f64().unwrap(), m.m2 / m.n.to_f64().unwrap()),
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
