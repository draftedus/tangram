use self::{
	classification_production_metrics::{
		ClassificationProductionPredictionMetrics, ClassificationProductionPredictionMetricsOutput,
	},
	regression_production_metrics::{
		RegressionProductionPredictionMetrics, RegressionProductionPredictionMetricsOutput,
	},
};
use crate::monitor_event::NumberOrString;
use chrono::prelude::*;
use tangram_core::metrics::Metric;

mod classification_production_metrics;
mod regression_production_metrics;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductionMetrics {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub true_values_count: u64,
	pub prediction_metrics: ProductionPredictionMetrics,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum ProductionPredictionMetrics {
	Classification(ClassificationProductionPredictionMetrics),
	Regression(RegressionProductionPredictionMetrics),
}

#[derive(Debug)]
pub struct ProductionMetricsOutput {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub true_values_count: u64,
	pub prediction_metrics: Option<ProductionPredictionMetricsOutput>,
}

#[derive(Debug)]
pub enum ProductionPredictionMetricsOutput {
	Regression(RegressionProductionPredictionMetricsOutput),
	Classification(ClassificationProductionPredictionMetricsOutput),
}

impl ProductionMetrics {
	pub fn new(
		model: &tangram_core::model::Model,
		start_date: DateTime<Utc>,
		end_date: DateTime<Utc>,
	) -> Self {
		let prediction_metrics = ProductionPredictionMetrics::new(model);
		Self {
			start_date,
			end_date,
			true_values_count: 0,
			prediction_metrics,
		}
	}
}

impl Metric<'_> for ProductionMetrics {
	type Input = (NumberOrString, NumberOrString);
	type Output = ProductionMetricsOutput;

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

impl ProductionPredictionMetrics {
	pub fn new(model: &tangram_core::model::Model) -> Self {
		match model {
			tangram_core::model::Model::Regressor(_) => {
				ProductionPredictionMetrics::Regression(RegressionProductionPredictionMetrics::new())
			}
			tangram_core::model::Model::Classifier(model) => {
				ProductionPredictionMetrics::Classification(
					ClassificationProductionPredictionMetrics::new(model.classes().to_owned()),
				)
			}
		}
	}
}

impl Metric<'_> for ProductionPredictionMetrics {
	type Input = (NumberOrString, NumberOrString);
	type Output = Option<ProductionPredictionMetricsOutput>;

	fn update(&mut self, value: (NumberOrString, NumberOrString)) {
		match self {
			ProductionPredictionMetrics::Classification(s) => s.update(value),
			ProductionPredictionMetrics::Regression(s) => s.update(value),
		}
	}

	fn merge(&mut self, other: Self) {
		match self {
			ProductionPredictionMetrics::Regression(s) => {
				if let ProductionPredictionMetrics::Regression(other) = other {
					s.merge(other)
				}
			}
			ProductionPredictionMetrics::Classification(s) => {
				if let ProductionPredictionMetrics::Classification(other) = other {
					s.merge(other)
				}
			}
		}
	}

	fn finalize(self) -> Self::Output {
		match self {
			ProductionPredictionMetrics::Classification(s) => match s.finalize() {
				Some(s) => Some(ProductionPredictionMetricsOutput::Classification(s)),
				None => None,
			},
			ProductionPredictionMetrics::Regression(s) => match s.finalize() {
				Some(s) => Some(ProductionPredictionMetricsOutput::Regression(s)),
				None => None,
			},
		}
	}
}
