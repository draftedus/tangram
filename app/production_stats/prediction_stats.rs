use super::NumberStats;
use crate::{monitor_event::Output, types};
use std::collections::BTreeMap as Map;
use tangram_core::metrics::RunningMetric;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", tag = "id", content = "value")]
pub enum PredictionStats {
	Regression(RegressionPredictionStats),
	Classification(ClassificationPredictionStats),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegressionPredictionStats {
	stats: Option<NumberStats>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationPredictionStats {
	pub histogram: Map<String, u64>,
}

impl PredictionStats {
	pub fn new(model: &tangram_core::types::Model) -> Self {
		match &model {
			tangram_core::types::Model::Regressor(_) => {
				PredictionStats::Regression(RegressionPredictionStats::new())
			}
			tangram_core::types::Model::Classifier(model) => {
				let classes = model.classes();
				PredictionStats::Classification(ClassificationPredictionStats::new(classes))
			}
			_ => unimplemented!(),
		}
	}
}

impl RunningMetric<'_, '_> for PredictionStats {
	type Input = Output;
	type Output = types::ProductionPredictionStats;

	fn update(&mut self, value: Self::Input) {
		match self {
			PredictionStats::Regression(stats) => stats.update(value),
			PredictionStats::Classification(stats) => stats.update(value),
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
			PredictionStats::Regression(stats) => {
				types::ProductionPredictionStats::Regression(stats.finalize())
			}
			PredictionStats::Classification(stats) => {
				types::ProductionPredictionStats::Classification(stats.finalize())
			}
		}
	}
}

impl RegressionPredictionStats {
	fn new() -> Self {
		Self { stats: None }
	}
}

impl RunningMetric<'_, '_> for RegressionPredictionStats {
	type Input = Output;
	type Output = types::RegressionProductionPredictionStats;

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

impl ClassificationPredictionStats {
	fn new(classes: &[String]) -> ClassificationPredictionStats {
		let histogram = classes.iter().cloned().map(|class| (class, 0)).collect();
		ClassificationPredictionStats { histogram }
	}
}

impl RunningMetric<'_, '_> for ClassificationPredictionStats {
	type Input = Output;
	type Output = types::ClassificationProductionPredictionStats;

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
