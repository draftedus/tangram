use self::{
	multiclass_classification_production_metrics::{
		MulticlassClassificationProductionPredictionMetrics,
		MulticlassClassificationProductionPredictionMetricsOutput,
	},
	regression_production_metrics::{
		RegressionProductionPredictionMetrics, RegressionProductionPredictionMetricsOutput,
	},
};
use crate::common::monitor_event::NumberOrString;
use chrono::prelude::*;
use tangram_metrics::StreamingMetric;

mod multiclass_classification_production_metrics;
mod regression_production_metrics;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProductionMetrics {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub true_values_count: u64,
	pub prediction_metrics: ProductionPredictionMetrics,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum ProductionPredictionMetrics {
	Regression(RegressionProductionPredictionMetrics),
	MulticlassClassification(MulticlassClassificationProductionPredictionMetrics),
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
	MulticlassClassification(MulticlassClassificationProductionPredictionMetricsOutput),
}

impl ProductionMetrics {
	pub fn new(
		model: &tangram_core::model::Model,
		start_date: DateTime<Utc>,
		end_date: DateTime<Utc>,
	) -> ProductionMetrics {
		let prediction_metrics = ProductionPredictionMetrics::new(model);
		ProductionMetrics {
			start_date,
			end_date,
			true_values_count: 0,
			prediction_metrics,
		}
	}
}

impl StreamingMetric<'_> for ProductionMetrics {
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
		ProductionMetricsOutput {
			start_date: self.start_date,
			end_date: self.end_date,
			true_values_count: self.true_values_count,
			prediction_metrics: self.prediction_metrics.finalize(),
		}
	}
}

impl ProductionPredictionMetrics {
	pub fn new(model: &tangram_core::model::Model) -> ProductionPredictionMetrics {
		match model {
			tangram_core::model::Model::Regressor(_) => {
				ProductionPredictionMetrics::Regression(RegressionProductionPredictionMetrics::new())
			}
			tangram_core::model::Model::BinaryClassifier(_) => todo!(),
			tangram_core::model::Model::MulticlassClassifier(model) => {
				ProductionPredictionMetrics::MulticlassClassification(
					MulticlassClassificationProductionPredictionMetrics::new(
						model.classes().to_owned(),
					),
				)
			}
		}
	}
}

impl StreamingMetric<'_> for ProductionPredictionMetrics {
	type Input = (NumberOrString, NumberOrString);
	type Output = Option<ProductionPredictionMetricsOutput>;

	fn update(&mut self, value: (NumberOrString, NumberOrString)) {
		match self {
			ProductionPredictionMetrics::MulticlassClassification(s) => s.update(value),
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
			ProductionPredictionMetrics::MulticlassClassification(s) => {
				if let ProductionPredictionMetrics::MulticlassClassification(other) = other {
					s.merge(other)
				}
			}
		}
	}

	fn finalize(self) -> Self::Output {
		match self {
			ProductionPredictionMetrics::MulticlassClassification(s) => match s.finalize() {
				Some(s) => Some(ProductionPredictionMetricsOutput::MulticlassClassification(
					s,
				)),
				None => None,
			},
			ProductionPredictionMetrics::Regression(s) => match s.finalize() {
				Some(s) => Some(ProductionPredictionMetricsOutput::Regression(s)),
				None => None,
			},
		}
	}
}
