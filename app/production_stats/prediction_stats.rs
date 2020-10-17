use super::number_stats::{NumberStats, NumberStatsOutput};
use crate::common::monitor_event::PredictOutput;
use std::collections::BTreeMap;
use tangram_metrics::StreamingMetric;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ProductionPredictionStats {
	Regression(RegressionProductionPredictionStats),
	BinaryClassification(ClassificationProductionPredictionStats),
	MulticlassClassification(ClassificationProductionPredictionStats),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct RegressionProductionPredictionStats {
	stats: Option<NumberStats>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ClassificationProductionPredictionStats {
	pub histogram: BTreeMap<String, u64>,
}

#[derive(Debug)]
pub enum ProductionPredictionStatsOutput {
	Regression(RegressionProductionPredictionStatsOutput),
	BinaryClassification(ClassificationProductionPredictionStatsOutput),
	MulticlassClassification(ClassificationProductionPredictionStatsOutput),
}

#[derive(Debug)]
pub struct RegressionProductionPredictionStatsOutput {
	pub stats: Option<NumberStatsOutput>,
}

#[derive(serde::Serialize, Debug)]
pub struct ClassificationProductionPredictionStatsOutput {
	pub histogram: Vec<(String, u64)>,
}

impl ProductionPredictionStats {
	pub fn new(model: &tangram_core::model::Model) -> ProductionPredictionStats {
		match model {
			tangram_core::model::Model::Regressor(_) => {
				ProductionPredictionStats::Regression(RegressionProductionPredictionStats::new())
			}
			tangram_core::model::Model::BinaryClassifier(model) => {
				let classes = model.classes();
				ProductionPredictionStats::BinaryClassification(
					ClassificationProductionPredictionStats::new(classes),
				)
			}
			tangram_core::model::Model::MulticlassClassifier(model) => {
				let classes = model.classes();
				ProductionPredictionStats::MulticlassClassification(
					ClassificationProductionPredictionStats::new(classes),
				)
			}
		}
	}
}

impl StreamingMetric<'_> for ProductionPredictionStats {
	type Input = PredictOutput;
	type Output = ProductionPredictionStatsOutput;

	fn update(&mut self, value: Self::Input) {
		match self {
			ProductionPredictionStats::Regression(stats) => stats.update(value),
			ProductionPredictionStats::BinaryClassification(stats) => stats.update(value),
			ProductionPredictionStats::MulticlassClassification(stats) => stats.update(value),
		}
	}

	fn merge(&mut self, other: Self) {
		match self {
			ProductionPredictionStats::Regression(this) => {
				if let ProductionPredictionStats::Regression(other) = other {
					this.merge(other)
				}
			}
			ProductionPredictionStats::BinaryClassification(this) => {
				if let ProductionPredictionStats::BinaryClassification(other) = other {
					this.merge(other)
				}
			}
			ProductionPredictionStats::MulticlassClassification(this) => {
				if let ProductionPredictionStats::MulticlassClassification(other) = other {
					this.merge(other)
				}
			}
		}
	}

	fn finalize(self) -> Self::Output {
		match self {
			ProductionPredictionStats::Regression(stats) => {
				ProductionPredictionStatsOutput::Regression(stats.finalize())
			}
			ProductionPredictionStats::BinaryClassification(stats) => {
				ProductionPredictionStatsOutput::BinaryClassification(stats.finalize())
			}
			ProductionPredictionStats::MulticlassClassification(stats) => {
				ProductionPredictionStatsOutput::MulticlassClassification(stats.finalize())
			}
		}
	}
}

impl RegressionProductionPredictionStats {
	fn new() -> RegressionProductionPredictionStats {
		RegressionProductionPredictionStats { stats: None }
	}
}

impl StreamingMetric<'_> for RegressionProductionPredictionStats {
	type Input = PredictOutput;
	type Output = RegressionProductionPredictionStatsOutput;

	fn update(&mut self, value: PredictOutput) {
		let value = match value {
			PredictOutput::Regression(value) => value,
			_ => unreachable!(),
		};
		match &mut self.stats {
			None => {
				self.stats.replace(NumberStats::new(value.value));
			}
			Some(stats) => stats.update(value.value),
		};
	}

	fn merge(&mut self, other: Self) {
		match &mut self.stats {
			None => self.stats = other.stats,
			Some(stats) => {
				if let Some(other) = other.stats {
					stats.merge(other)
				}
			}
		};
	}

	fn finalize(self) -> Self::Output {
		let stats = self.stats.map(|s| s.finalize());
		RegressionProductionPredictionStatsOutput { stats }
	}
}

impl ClassificationProductionPredictionStats {
	fn new(classes: &[String]) -> ClassificationProductionPredictionStats {
		let histogram = classes.iter().cloned().map(|class| (class, 0)).collect();
		ClassificationProductionPredictionStats { histogram }
	}
}

impl StreamingMetric<'_> for ClassificationProductionPredictionStats {
	type Input = PredictOutput;
	type Output = ClassificationProductionPredictionStatsOutput;

	fn update(&mut self, value: PredictOutput) {
		let value = match value {
			PredictOutput::MulticlassClassification(value) => value,
			_ => unreachable!(),
		};
		if let Some(count) = self.histogram.get_mut(&value.class_name) {
			*count += 1;
		}
	}

	fn merge(&mut self, other: Self) {
		for (value, count) in other.histogram.into_iter() {
			*self.histogram.entry(value).or_insert(0) += count;
		}
	}

	fn finalize(self) -> Self::Output {
		ClassificationProductionPredictionStatsOutput {
			histogram: self.histogram.into_iter().collect(),
		}
	}
}
