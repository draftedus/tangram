use super::number_stats::{NumberStats, NumberStatsOutput};
use itertools::Itertools;
use num_traits::ToPrimitive;
use serde::ser::SerializeSeq;
use std::collections::{BTreeMap, HashMap, HashSet};
use tangram_metrics::StreamingMetric;
use tangram_util::alphanumeric_tokenizer::AlphanumericTokenizer;

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
	#[serde(
		serialize_with = "serialize_token_histogram",
		deserialize_with = "deserialize_token_histogram"
	)]
	pub token_histogram: HashMap<Token, u64>,
	#[serde(
		serialize_with = "serialize_token_histogram",
		deserialize_with = "deserialize_token_histogram"
	)]
	pub per_example_histogram: HashMap<Token, u64>,
	pub tokenizer: Tokenizer,
}

fn serialize_token_histogram<S>(map: &HashMap<Token, u64>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: serde::Serializer,
{
	let mut seq = serializer.serialize_seq(Some(map.len()))?;
	for (key, value) in map.iter() {
		seq.serialize_element(&(key, value))?;
	}
	seq.end()
}

fn deserialize_token_histogram<'de, D>(deserializer: D) -> Result<HashMap<Token, u64>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	struct Visitor;
	impl<'de> serde::de::Visitor<'de> for Visitor {
		type Value = HashMap<Token, u64>;
		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str("Vec<(Token, u64)>")
		}

		fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
		where
			A: serde::de::SeqAccess<'de>,
		{
			let mut map = HashMap::new();
			while let Some((key, value)) = seq.next_element()? {
				map.insert(key, value);
			}
			Ok(map)
		}
	}
	deserializer.deserialize_seq(Visitor)
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
	Unigram(String),
	Bigram(String, String),
}

impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Token::Unigram(token) => write!(f, "{}", token),
			Token::Bigram(token_a, token_b) => write!(f, "{} {}", token_a, token_b),
		}
	}
}

impl From<tangram_core::model::Token> for Token {
	fn from(value: tangram_core::model::Token) -> Token {
		match value {
			tangram_core::model::Token::Unigram(token) => Token::Unigram(token),
			tangram_core::model::Token::Bigram(token_a, token_b) => Token::Bigram(token_a, token_b),
		}
	}
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
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
}

#[derive(Debug)]
pub struct NumberProductionColumnStatsOutput {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
	pub stats: Option<NumberStatsOutput>,
}

#[derive(Debug)]
pub struct EnumProductionColumnStatsOutput {
	pub absent_count: u64,
	pub column_name: String,
	pub histogram: Vec<(String, u64)>,
	pub invalid_count: u64,
	pub invalid_histogram: Option<Vec<(String, u64)>>,
}

#[derive(Debug)]
pub struct TextProductionColumnStatsOutput {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
	pub per_example_histogram: Vec<(Token, u64)>,
	pub token_histogram: Vec<(Token, u64)>,
}

impl ProductionColumnStats {
	pub fn new(column_stats: &tangram_core::model::ColumnStats) -> ProductionColumnStats {
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
			ProductionColumnStats::Unknown(s) => s.column_name.as_str(),
			ProductionColumnStats::Text(s) => s.column_name.as_str(),
			ProductionColumnStats::Number(s) => s.column_name.as_str(),
			ProductionColumnStats::Enum(s) => s.column_name.as_str(),
		}
	}
}

impl<'a> StreamingMetric<'a> for ProductionColumnStats {
	type Input = Option<&'a serde_json::Value>;
	type Output = ProductionColumnStatsOutput;

	fn update(&mut self, value: Self::Input) {
		match self {
			ProductionColumnStats::Unknown(stats) => stats.update(value),
			ProductionColumnStats::Text(stats) => stats.update(value),
			ProductionColumnStats::Number(stats) => stats.update(value),
			ProductionColumnStats::Enum(stats) => stats.update(value),
		}
	}

	fn merge(&mut self, other: Self) {
		match self {
			ProductionColumnStats::Unknown(stats) => {
				if let ProductionColumnStats::Unknown(other) = other {
					stats.merge(other)
				}
			}
			ProductionColumnStats::Text(stats) => {
				if let ProductionColumnStats::Text(other) = other {
					stats.merge(other)
				}
			}
			ProductionColumnStats::Number(stats) => {
				if let ProductionColumnStats::Number(other) = other {
					stats.merge(other)
				}
			}
			ProductionColumnStats::Enum(stats) => {
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
	fn new(column_stats: &tangram_core::model::UnknownColumnStats) -> UnknownProductionColumnStats {
		UnknownProductionColumnStats {
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
		UnknownProductionColumnStatsOutput {
			absent_count: self.absent_count,
			column_name: self.column_name,
			invalid_count: self.invalid_count,
		}
	}
}

impl NumberProductionColumnStats {
	fn new(column_stats: &tangram_core::model::NumberColumnStats) -> NumberProductionColumnStats {
		NumberProductionColumnStats {
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
				Ok(value) => value,
				Err(_) => {
					self.invalid_count += 1;
					return;
				}
			},
			Some(serde_json::Value::Number(value)) => match value.as_f64() {
				Some(value) => value.to_f32().unwrap(),
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
		NumberProductionColumnStatsOutput {
			absent_count: self.absent_count,
			column_name: self.column_name,
			invalid_count: self.invalid_count,
			stats: self.stats.map(|s| s.finalize()),
		}
	}
}

impl EnumProductionColumnStats {
	fn new(column_stats: &tangram_core::model::EnumColumnStats) -> EnumProductionColumnStats {
		let column_name = &column_stats.column_name;
		let histogram = column_stats
			.histogram
			.iter()
			.map(|(value, _)| (value.clone(), 0))
			.collect();
		EnumProductionColumnStats {
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
		EnumProductionColumnStatsOutput {
			absent_count: self.absent_count,
			column_name: self.column_name,
			histogram: self.histogram.into_iter().collect(),
			invalid_count: self.invalid_count,
			invalid_histogram: self.invalid_histogram.map(|h| h.into_iter().collect()),
		}
	}
}

impl TextProductionColumnStats {
	fn new(column_stats: &tangram_core::model::TextColumnStats) -> TextProductionColumnStats {
		let tokenizer = match column_stats.tokenizer {
			tangram_core::model::Tokenizer::Alphanumeric => Tokenizer::Alphanumeric,
		};
		let token_histogram = column_stats
			.top_tokens
			.iter()
			.map(|value| (value.token.clone().into(), 0))
			.collect();
		let per_example_histogram = column_stats
			.top_tokens
			.iter()
			.map(|value| (value.token.clone().into(), 0))
			.collect();
		TextProductionColumnStats {
			column_name: column_stats.column_name.clone(),
			absent_count: 0,
			invalid_count: 0,
			count: 0,
			token_histogram,
			per_example_histogram,
			tokenizer,
		}
	}
}

impl<'a> StreamingMetric<'a> for TextProductionColumnStats {
	type Input = Option<&'a serde_json::Value>;
	type Output = TextProductionColumnStatsOutput;

	fn update(&mut self, value: Self::Input) {
		self.count += 1;
		let value = match value {
			None => {
				self.absent_count += 1;
				return;
			}
			Some(serde_json::Value::String(value)) => value,
			_ => {
				self.invalid_count += 1;
				return;
			}
		};
		// Tokenize the value.
		match self.tokenizer {
			Tokenizer::Alphanumeric => {
				let mut token_set = HashSet::new();
				for unigram in AlphanumericTokenizer::new(value) {
					let unigram = Token::Unigram(unigram.into_owned());
					match self.token_histogram.get_mut(&unigram) {
						Some(count) => *count += 1,
						None => {
							self.invalid_count += 1;
						}
					};
					token_set.insert(unigram);
				}
				for bigram in AlphanumericTokenizer::new(value).tuple_windows::<(_, _)>() {
					let bigram = Token::Bigram(bigram.0.into_owned(), bigram.1.into_owned());
					match self.token_histogram.get_mut(&bigram) {
						Some(count) => *count += 1,
						None => {
							self.invalid_count += 1;
						}
					};
					token_set.insert(bigram);
				}
				for token in token_set.iter() {
					if let Some(count) = self.per_example_histogram.get_mut(token) {
						*count += 1;
					}
				}
			}
		}
	}

	fn merge(&mut self, other: Self) {
		self.invalid_count += other.invalid_count;
		self.absent_count += other.absent_count;
		self.count += other.count;
		for (value, count) in other.token_histogram.into_iter() {
			if let Some(entry) = self.token_histogram.get_mut(&value) {
				*entry += count;
			} else {
				self.token_histogram.insert(value, count);
			}
		}
		for (value, count) in other.per_example_histogram.into_iter() {
			if let Some(entry) = self.per_example_histogram.get_mut(&value) {
				*entry += count;
			} else {
				self.per_example_histogram.insert(value, count);
			}
		}
	}

	fn finalize(self) -> Self::Output {
		TextProductionColumnStatsOutput {
			absent_count: self.absent_count,
			column_name: self.column_name,
			invalid_count: self.invalid_count,
			per_example_histogram: self.per_example_histogram.into_iter().collect(),
			token_histogram: self.token_histogram.into_iter().collect(),
		}
	}
}

impl ProductionColumnStatsOutput {
	pub fn column_name(&self) -> &str {
		match self {
			ProductionColumnStatsOutput::Unknown(s) => s.column_name.as_str(),
			ProductionColumnStatsOutput::Text(s) => s.column_name.as_str(),
			ProductionColumnStatsOutput::Number(s) => s.column_name.as_str(),
			ProductionColumnStatsOutput::Enum(s) => s.column_name.as_str(),
		}
	}
}
