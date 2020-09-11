use super::number_stats::{NumberStats, NumberStatsOutput};
use crate::app::monitor_event::Output;
use std::collections::BTreeMap;
use tangram::metrics::Metric;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ProductionPredictionStats {
	Regression(RegressionProductionPredictionStats),
	Classification(ClassificationProductionPredictionStats),
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
	Classification(ClassificationProductionPredictionStatsOutput),
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
	pub fn new(model: &tangram::model::Model) -> Self {
		match &model {
			tangram::model::Model::Regressor(_) => {
				ProductionPredictionStats::Regression(RegressionProductionPredictionStats::new())
			}
			tangram::model::Model::Classifier(model) => {
				let classes = model.classes();
				ProductionPredictionStats::Classification(
					ClassificationProductionPredictionStats::new(classes),
				)
			}
		}
	}
}

impl Metric<'_> for ProductionPredictionStats {
	type Input = Output;
	type Output = ProductionPredictionStatsOutput;

	fn update(&mut self, value: Self::Input) {
		match self {
			ProductionPredictionStats::Regression(stats) => stats.update(value),
			ProductionPredictionStats::Classification(stats) => stats.update(value),
		}
	}

	fn merge(&mut self, other: Self) {
		match self {
			Self::Regression(this) => {
				if let Self::Regression(other) = other {
					this.merge(other)
				}
			}
			Self::Classification(this) => {
				if let Self::Classification(other) = other {
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
			ProductionPredictionStats::Classification(stats) => {
				ProductionPredictionStatsOutput::Classification(stats.finalize())
			}
		}
	}
}

impl RegressionProductionPredictionStats {
	fn new() -> Self {
		Self { stats: None }
	}
}

impl Metric<'_> for RegressionProductionPredictionStats {
	type Input = Output;
	type Output = RegressionProductionPredictionStatsOutput;

	fn update(&mut self, value: Output) {
		let value = match value {
			Output::Regression(value) => value,
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
		Self::Output { stats }
	}
}

impl ClassificationProductionPredictionStats {
	fn new(classes: &[String]) -> ClassificationProductionPredictionStats {
		let histogram = classes.iter().cloned().map(|class| (class, 0)).collect();
		ClassificationProductionPredictionStats { histogram }
	}
}

impl Metric<'_> for ClassificationProductionPredictionStats {
	type Input = Output;
	type Output = ClassificationProductionPredictionStatsOutput;

	fn update(&mut self, value: Output) {
		let value = match value {
			Output::Classification(value) => value,
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
		Self::Output {
			histogram: self.histogram.into_iter().collect(),
		}
	}
}
