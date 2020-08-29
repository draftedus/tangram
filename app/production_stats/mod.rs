use crate::monitor_event::PredictionMonitorEvent;
use chrono::prelude::*;
use tangram_core::metrics::RunningMetric;

mod column_stats;
mod number_stats;
mod prediction_stats;

pub use column_stats::*;
pub use number_stats::*;
pub use prediction_stats::*;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ProductionStats {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub row_count: u64,
	pub column_stats: Vec<ProductionColumnStats>,
	pub prediction_stats: ProductionPredictionStats,
}

#[derive(Debug)]
pub struct ProductionStatsOutput {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub row_count: u64,
	pub column_stats: Vec<ProductionColumnStatsOutput>,
	pub prediction_stats: ProductionPredictionStatsOutput,
}

impl ProductionStats {
	pub fn new(
		model: &tangram_core::types::Model,
		start_date: DateTime<Utc>,
		end_date: DateTime<Utc>,
	) -> Self {
		let train_column_stats = match &model {
			tangram_core::types::Model::Regressor(model) => model.train_column_stats.as_slice(),
			tangram_core::types::Model::Classifier(model) => model.train_column_stats.as_slice(),
		};
		let column_stats = train_column_stats
			.iter()
			.map(|column_stats| ProductionColumnStats::new(column_stats))
			.collect();
		let prediction_stats = ProductionPredictionStats::new(model);
		ProductionStats {
			start_date,
			end_date,
			row_count: 0,
			column_stats,
			prediction_stats,
		}
	}
}

impl RunningMetric<'_, '_> for ProductionStats {
	type Input = PredictionMonitorEvent;
	type Output = ProductionStatsOutput;

	fn update(&mut self, value: PredictionMonitorEvent) {
		self.row_count += 1;
		for column_stats in self.column_stats.iter_mut() {
			let value = value.input.get(column_stats.column_name());
			column_stats.update(value);
		}
		self.prediction_stats.update(value.output);
	}

	fn merge(&mut self, other: Self) {
		self.start_date = self.start_date.min(other.start_date);
		self.end_date = self.end_date.max(other.end_date);
		self.row_count += other.row_count;
		self.column_stats
			.iter_mut()
			.zip(other.column_stats.into_iter())
			.for_each(|(this, other)| this.merge(other));
		self.prediction_stats.merge(other.prediction_stats);
	}

	fn finalize(self) -> Self::Output {
		Self::Output {
			start_date: self.start_date,
			end_date: self.end_date,
			row_count: self.row_count,
			column_stats: self
				.column_stats
				.into_iter()
				.map(|c| c.finalize())
				.collect(),
			prediction_stats: self.prediction_stats.finalize(),
		}
	}
}
