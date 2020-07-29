use crate::app::{monitor_event::NumberOrString, types};
use chrono::prelude::*;
use tangram::metrics::RunningMetric;

mod classification_production_metrics;
mod regression_production_metrics;

pub use self::classification_production_metrics::*;
pub use self::regression_production_metrics::*;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductionMetrics {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub true_values_count: u64,
	pub prediction_metrics: PredictionMetrics,
}

impl ProductionMetrics {
	pub fn new(
		model: &tangram::types::Model,
		start_date: DateTime<Utc>,
		end_date: DateTime<Utc>,
	) -> Self {
		let prediction_metrics = PredictionMetrics::new(model);
		Self {
			start_date,
			end_date,
			true_values_count: 0,
			prediction_metrics,
		}
	}
}

impl RunningMetric<'_, '_> for ProductionMetrics {
	type Input = (NumberOrString, NumberOrString);
	type Output = types::ProductionMetrics;

	fn update(&mut self, value: (NumberOrString, NumberOrString)) {
		self.true_values_count += 1;
		self.prediction_metrics.update(value);
	}

	fn merge(&mut self, other: Self) {
		self.start_date = self.start_date.min(other.start_date);
		self.end_date = self.end_date.max(other.end_date);
		self.prediction_metrics.merge(other.prediction_metrics);
		self.true_values_count += other.true_values_count;
	}

	fn finalize(self) -> Self::Output {
		Self::Output {
			start_date: self.start_date,
			end_date: self.end_date,
			true_values_count: self.true_values_count,
			prediction_metrics: self.prediction_metrics.finalize(),
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum PredictionMetrics {
	Classification(ClassificationPredictionMetrics),
	Regression(RegressionPredictionMetrics),
}

impl PredictionMetrics {
	pub fn new(model: &tangram::types::Model) -> Self {
		match model {
			tangram::types::Model::Regressor(_) => {
				PredictionMetrics::Regression(RegressionPredictionMetrics::new())
			}
			tangram::types::Model::Classifier(model) => PredictionMetrics::Classification(
				ClassificationPredictionMetrics::new(model.classes().to_owned()),
			),
			_ => unimplemented!(),
		}
	}
}

impl RunningMetric<'_, '_> for PredictionMetrics {
	type Input = (NumberOrString, NumberOrString);
	type Output = Option<types::PredictionMetrics>;

	fn update(&mut self, value: (NumberOrString, NumberOrString)) {
		match self {
			PredictionMetrics::Classification(s) => s.update(value),
			PredictionMetrics::Regression(s) => s.update(value),
		}
	}

	fn merge(&mut self, other: Self) {
		match self {
			PredictionMetrics::Regression(s) => {
				if let PredictionMetrics::Regression(other) = other {
					s.merge(other)
				}
			}
			PredictionMetrics::Classification(s) => {
				if let PredictionMetrics::Classification(other) = other {
					s.merge(other)
				}
			}
		}
	}

	fn finalize(self) -> Self::Output {
		match self {
			PredictionMetrics::Classification(s) => match s.finalize() {
				Some(s) => Some(types::PredictionMetrics::Classification(s)),
				None => None,
			},
			PredictionMetrics::Regression(s) => match s.finalize() {
				Some(s) => Some(types::PredictionMetrics::Regression(s)),
				None => None,
			},
		}
	}
}
