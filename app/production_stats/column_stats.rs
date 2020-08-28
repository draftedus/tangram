use super::number_stats::{NumberStats, NumberStatsOutput};
use num_traits::ToPrimitive;
use std::collections::BTreeMap;
use tangram_core::metrics::RunningMetric;

const LARGE_ABSENT_RATIO_THRESHOLD: f32 = 0.1;
const LARGE_INVALID_RATIO_THRESHOLD: f32 = 0.1;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", tag = "id", content = "value")]
pub enum ProductionColumnStats {
	Unknown(UnknownProductionColumnStats),
	Number(NumberProductionColumnStats),
	Enum(EnumProductionColumnStats),
	Text(TextProductionColumnStats),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnknownProductionColumnStats {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
	pub count: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberProductionColumnStats {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
	pub stats: Option<NumberStats>,
	pub count: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnumProductionColumnStats {
	pub absent_count: u64,
	pub column_name: String,
	pub histogram: BTreeMap<String, u64>,
	pub invalid_count: u64,
	pub invalid_histogram: Option<BTreeMap<String, u64>>,
	pub count: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TextProductionColumnStats {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
	pub count: u64,
	pub token_histogram: BTreeMap<String, u64>,
	// pub tokenizer: Tokenizer,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Tokenizer {
	Alphanumeric,
}

#[derive(Debug)]
pub enum ProductionColumnStatsOutput {
	Unknown(UnknownProductionColumnStatsOutput),
	Number(NumberProductionColumnStatsOutput),
	Enum(EnumProductionColumnStatsOutput),
	Text(TextProductionColumnStatsOutput),
}

#[derive(Debug)]
pub struct UnknownProductionColumnStatsOutput {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub alert: Option<String>,
}

#[derive(Debug)]
pub struct NumberProductionColumnStatsOutput {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub stats: Option<NumberStatsOutput>,
	pub alert: Option<String>,
}

#[derive(Debug)]
pub struct EnumProductionColumnStatsOutput {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub histogram: Vec<(String, u64)>,
	pub invalid_histogram: Option<Vec<(String, u64)>>,
	pub alert: Option<String>,
}

#[derive(Debug)]
pub struct TextProductionColumnStatsOutput {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub alert: Option<String>,
	pub token_histogram: Vec<(String, u64)>,
}

impl ProductionColumnStats {
	pub fn new(column_stats: &tangram_core::types::ColumnStats) -> Self {
		match column_stats {
			tangram_core::types::ColumnStats::Unknown(stats) => {
				ProductionColumnStats::Unknown(UnknownProductionColumnStats::new(stats))
			}
			tangram_core::types::ColumnStats::Text(stats) => {
				ProductionColumnStats::Text(TextProductionColumnStats::new(stats))
			}
			tangram_core::types::ColumnStats::Number(stats) => {
				ProductionColumnStats::Number(NumberProductionColumnStats::new(stats))
			}
			tangram_core::types::ColumnStats::Enum(stats) => {
				ProductionColumnStats::Enum(EnumProductionColumnStats::new(stats))
			}
			_ => unimplemented!(),
		}
	}

	pub fn column_name(&self) -> &str {
		match self {
			Self::Unknown(s) => s.column_name.as_str(),
			Self::Text(s) => s.column_name.as_str(),
			Self::Number(s) => s.column_name.as_str(),
			Self::Enum(s) => s.column_name.as_str(),
		}
	}
}

impl<'a> RunningMetric<'a, '_> for ProductionColumnStats {
	type Input = Option<&'a serde_json::Value>;
	type Output = ProductionColumnStatsOutput;

	fn update(&mut self, value: Self::Input) {
		match self {
			Self::Unknown(stats) => stats.update(value),
			Self::Text(stats) => stats.update(value),
			Self::Number(stats) => stats.update(value),
			Self::Enum(stats) => stats.update(value),
		}
	}

	fn merge(&mut self, other: Self) {
		match self {
			Self::Unknown(stats) => {
				if let ProductionColumnStats::Unknown(other) = other {
					stats.merge(other)
				}
			}
			Self::Text(stats) => {
				if let ProductionColumnStats::Text(other) = other {
					stats.merge(other)
				}
			}
			Self::Number(stats) => {
				if let ProductionColumnStats::Number(other) = other {
					stats.merge(other)
				}
			}
			Self::Enum(stats) => {
				if let ProductionColumnStats::Enum(other) = other {
					stats.merge(other)
				}
			}
		}
	}

	fn finalize(self) -> Self::Output {
		match self {
			ProductionColumnStats::Unknown(stats) => {
				ProductionColumnStatsOutput::Unknown(stats.finalize())
			}
			ProductionColumnStats::Text(stats) => {
				ProductionColumnStatsOutput::Text(stats.finalize())
			}
			ProductionColumnStats::Number(stats) => {
				ProductionColumnStatsOutput::Number(stats.finalize())
			}
			ProductionColumnStats::Enum(stats) => {
				ProductionColumnStatsOutput::Enum(stats.finalize())
			}
		}
	}
}

impl UnknownProductionColumnStats {
	fn new(column_stats: &tangram_core::types::UnknownColumnStats) -> Self {
		Self {
			column_name: column_stats.column_name.as_option().unwrap().clone(),
			invalid_count: 0,
			absent_count: 0,
			count: 0,
		}
	}
}

impl<'a> RunningMetric<'a, '_> for UnknownProductionColumnStats {
	type Input = Option<&'a serde_json::Value>;
	type Output = UnknownProductionColumnStatsOutput;

	fn update(&mut self, value: Self::Input) {
		self.count += 1;
		match value {
			None => {
				self.absent_count += 1;
			}
			Some(serde_json::Value::Null) => {
				self.invalid_count += 1;
			}
			Some(serde_json::Value::String(value)) if value == "" => {
				self.invalid_count += 1;
			}
			_ => self.invalid_count += 1,
		};
	}

	fn merge(&mut self, other: Self) {
		self.absent_count += other.absent_count;
		self.invalid_count += other.invalid_count;
		self.count += other.count;
	}

	fn finalize(self) -> Self::Output {
		let invalid_ratio = self.invalid_count.to_f32().unwrap() / self.count.to_f32().unwrap();
		let absent_ratio = self.absent_count.to_f32().unwrap() / self.count.to_f32().unwrap();
		let alert = alert_message(invalid_ratio, absent_ratio);
		Self::Output {
			column_name: self.column_name,
			absent_count: self.absent_count,
			invalid_count: self.invalid_count,
			alert,
		}
	}
}

impl NumberProductionColumnStats {
	fn new(column_stats: &tangram_core::types::NumberColumnStats) -> Self {
		Self {
			column_name: column_stats.column_name.as_option().unwrap().clone(),
			absent_count: 0,
			invalid_count: 0,
			stats: None,
			count: 0,
		}
	}
}

impl<'a, 'b> RunningMetric<'a, 'b> for NumberProductionColumnStats {
	type Input = Option<&'a serde_json::Value>;
	type Output = NumberProductionColumnStatsOutput;

	fn update(&mut self, value: Self::Input) {
		self.count += 1;
		let value = match value {
			None => {
				self.absent_count += 1;
				return;
			}
			Some(serde_json::Value::String(value)) => match lexical::parse::<f32, _>(value) {
				Ok(n) => n,
				Err(_) => {
					self.invalid_count += 1;
					return;
				}
			},
			Some(serde_json::Value::Number(value)) => match value.as_f64() {
				Some(n) => n.to_f32().unwrap(),
				None => {
					self.invalid_count += 1;
					return;
				}
			},
			Some(serde_json::Value::Bool(_)) => {
				self.invalid_count += 1;
				return;
			}
			Some(serde_json::Value::Null) => {
				self.invalid_count += 1;
				return;
			}
			_ => {
				self.invalid_count += 1;
				return;
			}
		};
		match &mut self.stats {
			Some(stats) => stats.update(value),
			None => {
				self.stats.replace(NumberStats::new(value));
			}
		};
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
		self.absent_count += other.absent_count;
		self.invalid_count += other.invalid_count;
		self.count += other.count;
	}

	fn finalize(self) -> Self::Output {
		let invalid_ratio = self.invalid_count.to_f32().unwrap() / self.count.to_f32().unwrap();
		let absent_ratio = self.absent_count.to_f32().unwrap() / self.count.to_f32().unwrap();
		let alert = alert_message(invalid_ratio, absent_ratio);
		let stats = self.stats.map(|s| s.finalize());
		Self::Output {
			column_name: self.column_name,
			absent_count: self.absent_count,
			invalid_count: self.invalid_count,
			stats,
			alert,
		}
	}
}

impl EnumProductionColumnStats {
	fn new(column_stats: &tangram_core::types::EnumColumnStats) -> Self {
		let column_name = column_stats.column_name.as_option().unwrap();
		let histogram = column_stats
			.histogram
			.as_option()
			.unwrap()
			.iter()
			.map(|(value, _)| (value.clone(), 0))
			.collect();
		Self {
			column_name: column_name.clone(),
			invalid_count: 0,
			absent_count: 0,
			histogram,
			invalid_histogram: None,
			count: 0,
		}
	}
}

impl<'a> RunningMetric<'a, '_> for EnumProductionColumnStats {
	type Input = Option<&'a serde_json::Value>;
	type Output = EnumProductionColumnStatsOutput;

	fn update(&mut self, value: Self::Input) {
		self.count += 1;
		let value = match value {
			None => {
				self.absent_count += 1;
				return;
			}
			Some(serde_json::Value::Number(_)) => {
				self.invalid_count += 1;
				return;
			}
			Some(serde_json::Value::Bool(true)) => "true",
			Some(serde_json::Value::Bool(false)) => "false",
			Some(serde_json::Value::String(value)) => value,
			Some(serde_json::Value::Null) => {
				self.invalid_count += 1;
				return;
			}
			_ => {
				self.invalid_count += 1;
				return;
			}
		};
		match self.histogram.get_mut(value) {
			Some(count) => *count += 1,
			None => {
				self.invalid_count += 1;
				match &mut self.invalid_histogram {
					Some(histogram) => match histogram.get_mut(value) {
						Some(count) => *count += 1,
						None => {
							histogram.insert(value.into(), 1);
						}
					},
					None => {
						let mut invalid_histogram: BTreeMap<String, u64> = BTreeMap::new();
						invalid_histogram.insert(value.into(), 1);
						self.invalid_histogram = Some(invalid_histogram)
					}
				}
			}
		};
	}

	fn merge(&mut self, other: Self) {
		self.invalid_count += other.invalid_count;
		self.absent_count += other.absent_count;
		for (value, count) in other.histogram.into_iter() {
			*self.histogram.entry(value).or_insert(0) += count;
		}
		self.count += other.count;
		match &mut self.invalid_histogram {
			Some(histogram) => {
				if let Some(other) = other.invalid_histogram {
					for (value, count) in other.into_iter() {
						*histogram.entry(value).or_insert(0) += count;
					}
				};
			}
			None => {
				if let Some(other) = other.invalid_histogram {
					self.invalid_histogram = Some(other);
				}
			}
		}
	}

	fn finalize(self) -> Self::Output {
		let invalid_ratio = self.invalid_count.to_f32().unwrap() / self.count.to_f32().unwrap();
		let absent_ratio = self.absent_count.to_f32().unwrap() / self.count.to_f32().unwrap();
		let alert = alert_message(invalid_ratio, absent_ratio);
		Self::Output {
			column_name: self.column_name,
			histogram: self.histogram.into_iter().collect(),
			absent_count: self.absent_count,
			invalid_count: self.invalid_count,
			invalid_histogram: self.invalid_histogram.map(|h| h.into_iter().collect()),
			alert,
		}
	}
}

impl TextProductionColumnStats {
	fn new(column_stats: &tangram_core::types::TextColumnStats) -> Self {
		// let tokenizer = match feature_group {
		// 	tangram_core::types::FeatureGroup::BagOfWords(feature_group) => {
		// 		match feature_group.tokenizer.as_option().unwrap() {
		// 			tangram_core::types::Tokenizer::Alphanumeric => Tokenizer::Alphanumeric,
		// 			tangram_core::types::Tokenizer::UnknownVariant(_, _, _) => unimplemented!(),
		// 		}
		// 	}
		// 	tangram_core::types::FeatureGroup::Identity(_) => unreachable!(),
		// 	tangram_core::types::FeatureGroup::Normalized(_) => unreachable!(),
		// 	tangram_core::types::FeatureGroup::OneHotEncoded(_) => unreachable!(),
		// 	_ => unimplemented!(),
		// };
		Self {
			column_name: column_stats.column_name.as_option().unwrap().clone(),
			absent_count: 0,
			invalid_count: 0,
			count: 0,
			token_histogram: BTreeMap::new(),
			// tokenizer,
		}
	}
}

impl<'a> RunningMetric<'a, '_> for TextProductionColumnStats {
	type Input = Option<&'a serde_json::Value>;
	type Output = TextProductionColumnStatsOutput;

	fn update(&mut self, value: Self::Input) {
		self.count += 1;
		match value {
			None => {
				self.absent_count += 1;
			}
			Some(serde_json::Value::String(_)) => {}
			_ => {
				self.invalid_count += 1;
			}
		};
		// if let Some(serde_json::Value::String(value)) = value {
		// match self.tokenizer {
		// 	Tokenizer::Alphanumeric => {
		// 		let tokenizer = tangram_core::util::text::AlphanumericTokenizer;
		// 		let tokens = tokenizer.tokenize(value);
		// 		let bigrams = tangram_core::util::text::bigrams(&tokens);
		// 		for token in tokens.iter().chain(bigrams.iter()) {
		// 			// insert the token into the histogram
		// 			match self.token_histogram.get_mut(token) {
		// 				Some(count) => *count += 1,
		// 				None => {
		// 					self.token_histogram.insert(value.into(), 1);
		// 				}
		// 			}
		// 		}
		// 	}
		// }
		// }
	}

	fn merge(&mut self, other: Self) {
		self.invalid_count += other.invalid_count;
		self.absent_count += other.absent_count;
		self.count += other.count;
		for (value, count) in other.token_histogram.into_iter() {
			*self.token_histogram.entry(value).or_insert(0) += count;
		}
	}

	fn finalize(self) -> Self::Output {
		let invalid_ratio = self.invalid_count.to_f32().unwrap() / self.count.to_f32().unwrap();
		let absent_ratio = self.absent_count.to_f32().unwrap() / self.count.to_f32().unwrap();
		let alert = alert_message(invalid_ratio, absent_ratio);
		Self::Output {
			column_name: self.column_name,
			absent_count: self.absent_count,
			invalid_count: self.invalid_count,
			alert,
			token_histogram: self.token_histogram.into_iter().collect(),
		}
	}
}

fn alert_message(invalid_ratio: f32, absent_ratio: f32) -> Option<String> {
	if invalid_ratio > LARGE_INVALID_RATIO_THRESHOLD {
		if absent_ratio > LARGE_ABSENT_RATIO_THRESHOLD {
			Some("High Invalid and Absent Count".into())
		} else {
			Some("High Invalid Count".into())
		}
	} else if absent_ratio > LARGE_ABSENT_RATIO_THRESHOLD {
		Some("High Absent Count".into())
	} else {
		None
	}
}
