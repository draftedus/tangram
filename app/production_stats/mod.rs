use self::{column_stats::ProductionColumnStats, prediction_stats::PredictionStats};
use crate::{monitor_event::PredictionMonitorEvent, types};
use chrono::prelude::*;
use num_traits::ToPrimitive;
use rand::random;
use tangram_core::metrics::{self, RunningMetric};

mod column_stats;
mod prediction_stats;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductionStats {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub row_count: u64,
	pub column_stats: Vec<ProductionColumnStats>,
	pub prediction_stats: PredictionStats,
}

impl ProductionStats {
	pub fn new(
		model: &tangram_core::types::Model,
		start_date: DateTime<Utc>,
		end_date: DateTime<Utc>,
	) -> Self {
		let (train_column_stats, feature_groups) = match &model {
			tangram_core::types::Model::Regressor(model) => {
				let feature_groups = match model.model.as_option().unwrap() {
					tangram_core::types::RegressionModel::Linear(model) => {
						model.feature_groups.as_option().unwrap()
					}
					tangram_core::types::RegressionModel::Gbt(model) => {
						model.feature_groups.as_option().unwrap()
					}
					_ => unimplemented!(),
				};
				let train_column_stats = model.train_column_stats.as_option().unwrap().as_slice();
				(train_column_stats, feature_groups)
			}
			tangram_core::types::Model::Classifier(model) => {
				let feature_groups = match model.model.as_option().unwrap() {
					tangram_core::types::ClassificationModel::LinearBinary(model) => {
						model.feature_groups.as_option().unwrap()
					}
					tangram_core::types::ClassificationModel::GbtBinary(model) => {
						model.feature_groups.as_option().unwrap()
					}
					tangram_core::types::ClassificationModel::LinearMulticlass(model) => {
						model.feature_groups.as_option().unwrap()
					}
					tangram_core::types::ClassificationModel::GbtMulticlass(model) => {
						model.feature_groups.as_option().unwrap()
					}
					_ => unimplemented!(),
				};
				let train_column_stats = model.train_column_stats.as_option().unwrap().as_slice();
				(train_column_stats, feature_groups)
			}
			_ => unimplemented!(),
		};
		let column_stats = train_column_stats
			.iter()
			.zip(feature_groups.iter())
			.map(|(c, f)| ProductionColumnStats::new(f, c))
			.collect();
		let prediction_stats = PredictionStats::new(model);
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
	type Output = types::ProductionStats;

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
			predictions_count: self.row_count,
			column_stats: self
				.column_stats
				.into_iter()
				.map(|c| c.finalize())
				.collect(),
			prediction_stats: self.prediction_stats.finalize(),
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberStats {
	pub n: u64,
	pub min: f32,
	pub max: f32,
	pub mean: f64,
	pub m2: f64,
	pub reservoir: Vec<f32>,
	pub reservoir_size: usize,
}

impl NumberStats {
	pub fn new(value: f32) -> Self {
		Self {
			n: 1,
			min: value,
			max: value,
			mean: value as f64,
			m2: 0.0,
			reservoir: vec![value],
			reservoir_size: 100,
		}
	}
}

impl RunningMetric<'_, '_> for NumberStats {
	type Input = f32;
	type Output = types::NumberStats;

	fn update(&mut self, value: Self::Input) {
		let (new_mean, new_m2) =
			metrics::merge_mean_m2(self.n, self.mean, self.m2, 1, value as f64, 0.0);
		self.n += 1;
		self.mean = new_mean;
		self.m2 = new_m2;
		self.min = f32::min(self.min, value);
		self.max = f32::max(self.max, value);
		if self.reservoir.len() < self.reservoir_size {
			self.reservoir.push(value)
		} else {
			let index = (random::<f32>() * self.n.to_f32().unwrap())
				.floor()
				.to_usize()
				.unwrap();
			if index < self.reservoir_size {
				self.reservoir[index] = value;
			}
		}
	}

	fn merge(&mut self, other: Self) {
		let (new_mean, new_m2) =
			metrics::merge_mean_m2(self.n, self.mean, self.m2, other.n, other.mean, other.m2);
		self.n += other.n;
		self.mean = new_mean;
		self.m2 = new_m2;
		self.min = f32::min(self.min, other.min);
		self.max = f32::max(self.max, other.max);
		self.reservoir.extend(other.reservoir);
	}

	fn finalize(self) -> Self::Output {
		let reservoir_len = self.reservoir.len().to_f32().unwrap();
		let quantiles: Vec<f32> = vec![0.25, 0.50, 0.75];
		// find the index of each quantile given the total number of values in the dataset
		let quantile_indexes: Vec<usize> = quantiles
			.iter()
			.map(|q| ((reservoir_len - 1.0) * q).trunc().to_usize().unwrap())
			.collect();
		// the fractiononal part of the index
		// used to interpolate values if the index is not an integer value
		let quantile_fracts: Vec<f32> = quantiles
			.iter()
			.map(|q| ((reservoir_len - 1.0) * q).fract())
			.collect();
		let mut quantiles: Vec<f32> = vec![0.0; quantiles.len()];
		let mut samples = self.reservoir;
		samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
		quantiles
			.iter_mut()
			.zip(quantile_indexes.iter().zip(quantile_fracts))
			.for_each(|(quantile, (index, fract))| {
				let value = samples[*index];
				if fract > 0.0 {
					let next_value = samples[index + 1];
					// interpolate between two values
					*quantile = value * (1.0 - fract) + next_value * fract;
				} else {
					*quantile = value;
				}
			});
		Self::Output {
			n: self.n,
			p25: quantiles[0],
			p50: quantiles[1],
			p75: quantiles[2],
			mean: self.mean as f32,
			variance: metrics::m2_to_variance(self.m2, self.n),
			std: metrics::m2_to_variance(self.m2, self.n).sqrt(),
			min: self.min,
			max: self.max,
		}
	}
}
