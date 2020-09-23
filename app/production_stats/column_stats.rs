use super::number_stats::{NumberStats, NumberStatsOutput};
use num_traits::ToPrimitive;
use std::collections::BTreeMap;
use tangram_metrics::StreamingMetric;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ProductionColumnStats {
	Unknown(UnknownProductionColumnStats),
	Number(NumberProductionColumnStats),
	Enum(EnumProductionColumnStats),
	Text(TextProductionColumnStats),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct UnknownProductionColumnStats {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
	pub count: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct NumberProductionColumnStats {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
	pub stats: Option<NumberStats>,
	pub count: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct EnumProductionColumnStats {
	pub absent_count: u64,
	pub column_name: String,
	pub histogram: BTreeMap<String, u64>,
	pub invalid_count: u64,
	pub invalid_histogram: Option<BTreeMap<String, u64>>,
	pub count: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TextProductionColumnStats {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
	pub count: u64,
	pub token_histogram: BTreeMap<String, u64>,
	// pub tokenizer: Tokenizer,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
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
}

#[derive(Debug)]
pub struct NumberProductionColumnStatsOutput {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub stats: Option<NumberStatsOutput>,
}

#[derive(Debug)]
pub struct EnumProductionColumnStatsOutput {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub histogram: Vec<(String, u64)>,
	pub invalid_histogram: Option<Vec<(String, u64)>>,
}

#[derive(Debug)]
pub struct TextProductionColumnStatsOutput {
	pub column_name: String,
	pub absent_count: u64,
	pub invalid_count: u64,
	pub token_histogram: Vec<(String, u64)>,
}

impl ProductionColumnStats {
	pub fn new(column_stats: &tangram_core::model::ColumnStats) -> Self {
		match column_stats {
			tangram_core::model::ColumnStats::Unknown(stats) => {
				ProductionColumnStats::Unknown(UnknownProductionColumnStats::new(stats))
			}
			tangram_core::model::ColumnStats::Text(stats) => {
				ProductionColumnStats::Text(TextProductionColumnStats::new(stats))
			}
			tangram_core::model::ColumnStats::Number(stats) => {
				ProductionColumnStats::Number(NumberProductionColumnStats::new(stats))
			}
			tangram_core::model::ColumnStats::Enum(stats) => {
				ProductionColumnStats::Enum(EnumProductionColumnStats::new(stats))
			}
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

impl<'a> StreamingMetric<'a> for ProductionColumnStats {
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
	fn new(column_stats: &tangram_core::model::UnknownColumnStats) -> Self {
		Self {
			column_name: column_stats.column_name.clone(),
			invalid_count: 0,
			absent_count: 0,
			count: 0,
		}
	}
}

impl<'a> StreamingMetric<'a> for UnknownProductionColumnStats {
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
		Self::Output {
			column_name: self.column_name,
			absent_count: self.absent_count,
			invalid_count: self.invalid_count,
		}
	}
}

impl NumberProductionColumnStats {
	fn new(column_stats: &tangram_core::model::NumberColumnStats) -> Self {
		Self {
			column_name: column_stats.column_name.clone(),
			absent_count: 0,
			invalid_count: 0,
			stats: None,
			count: 0,
		}
	}
}

impl<'a> StreamingMetric<'a> for NumberProductionColumnStats {
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
		Self::Output {
			column_name: self.column_name,
			absent_count: self.absent_count,
			invalid_count: self.invalid_count,
			stats: self.stats.map(|s| s.finalize()),
		}
	}
}

impl EnumProductionColumnStats {
	fn new(column_stats: &tangram_core::model::EnumColumnStats) -> Self {
		let column_name = &column_stats.column_name;
		let histogram = column_stats
			.histogram
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

impl<'a> StreamingMetric<'a> for EnumProductionColumnStats {
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
		Self::Output {
			column_name: self.column_name,
			histogram: self.histogram.into_iter().collect(),
			absent_count: self.absent_count,
			invalid_count: self.invalid_count,
			invalid_histogram: self.invalid_histogram.map(|h| h.into_iter().collect()),
		}
	}
}

impl TextProductionColumnStats {
	fn new(column_stats: &tangram_core::model::TextColumnStats) -> Self {
		// let tokenizer = match feature_group {
		// 	tangram_core::model::FeatureGroup::BagOfWords(feature_group) => {
		// 		match feature_group.tokenizer {
		// 			tangram_core::model::Tokenizer::Alphanumeric => Tokenizer::Alphanumeric,
		// 		}
		// 	}
		// 	tangram_core::model::FeatureGroup::Identity(_) => unreachable!(),
		// 	tangram_core::model::FeatureGroup::Normalized(_) => unreachable!(),
		// 	tangram_core::model::FeatureGroup::OneHotEncoded(_) => unreachable!(),
		// 	_ => unimplemented!(),
		// };
		Self {
			column_name: column_stats.column_name.clone(),
			absent_count: 0,
			invalid_count: 0,
			count: 0,
			token_histogram: BTreeMap::new(),
			// tokenizer,
		}
	}
}

impl<'a> StreamingMetric<'a> for TextProductionColumnStats {
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
		// TODO collect production token stats
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
		Self::Output {
			column_name: self.column_name,
			absent_count: self.absent_count,
			invalid_count: self.invalid_count,
			token_histogram: self.token_histogram.into_iter().collect(),
		}
	}
}

impl ProductionColumnStatsOutput {
	pub fn column_name(&self) -> &str {
		match self {
			Self::Unknown(s) => s.column_name.as_str(),
			Self::Text(s) => s.column_name.as_str(),
			Self::Number(s) => s.column_name.as_str(),
			Self::Enum(s) => s.column_name.as_str(),
		}
	}
}
