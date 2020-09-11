use crate::app::{monitor_event::NumberOrString, production_stats::NumberStats};
use num_traits::ToPrimitive;
use tangram::metrics::Metric;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RegressionProductionPredictionMetrics {
	stats: Option<NumberStats>,
	absolute_error: f64,
	squared_error: f64,
}

#[derive(Debug)]
pub struct RegressionProductionPredictionMetricsOutput {
	pub mse: f32,
	pub rmse: f32,
	pub mae: f32,
	pub r2: f32,
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
}

impl RegressionProductionPredictionMetrics {
	pub fn new() -> Self {
		Self {
			stats: None,
			absolute_error: 0.0,
			squared_error: 0.0,
		}
	}
}

impl Metric<'_> for RegressionProductionPredictionMetrics {
	type Input = (NumberOrString, NumberOrString);
	type Output = Option<RegressionProductionPredictionMetricsOutput>;

	fn update(&mut self, value: Self::Input) {
		let prediction = match value.0.as_number() {
			Ok(value) => value,
			Err(_) => return,
		};
		let label = match value.1.as_number() {
			Ok(value) => value,
			Err(_) => return,
		};
		let absolute_error = prediction - label;
		let squared_error = absolute_error * absolute_error;
		match &mut self.stats {
			Some(stats) => stats.update(prediction),
			None => {
				self.stats.replace(NumberStats::new(prediction));
			}
		};
		self.absolute_error += absolute_error.to_f64().unwrap();
		self.squared_error += squared_error.to_f64().unwrap();
	}

	fn merge(&mut self, other: Self) {
		match &mut self.stats {
			Some(stats) => {
				if let Some(other) = other.stats {
					stats.merge(other)
				}
			}
			None => self.stats = other.stats,
		};
		self.absolute_error += other.absolute_error;
		self.squared_error += other.squared_error;
	}

	fn finalize(self) -> Self::Output {
		let stats = self.stats.map(|s| s.finalize());
		match stats {
			Some(stats) => {
				let variance = stats.variance;
				let mae = self.absolute_error.to_f32().unwrap() / stats.n.to_f32().unwrap();
				let mse = self.squared_error.to_f32().unwrap() / stats.n.to_f32().unwrap();
				let rmse = mse.sqrt();
				let r2 = 1.0
					- self.squared_error.to_f32().unwrap() / (variance * stats.n.to_f32().unwrap()); // Sum of Squared Error = variance * n
				let baseline_mse = variance;
				let baseline_rmse = baseline_mse.sqrt();
				Some(RegressionProductionPredictionMetricsOutput {
					mae,
					mse,
					r2,
					rmse,
					baseline_mse,
					baseline_rmse,
				})
			}
			None => None,
		}
	}
}
