use crate::stats;
use num_traits::ToPrimitive;
use std::{
	cmp::Ordering,
	collections::{BTreeMap, BTreeSet},
	num::NonZeroU64,
};
use tangram_dataframe::prelude::*;
use tangram_metrics as metrics;
use tangram_util::{alphanumeric_tokenizer::AlphanumericTokenizer, finite::Finite};

/// This struct holds column stats.
#[derive(Clone, Debug)]
pub struct Stats(pub Vec<ColumnStats>);

/// This is an enum describing the different types of stats where the type matches the type of the source column.
#[derive(Clone, Debug)]
pub enum ColumnStats {
	Unknown(UnknownColumnStats),
	Number(NumberColumnStats),
	Enum(EnumColumnStats),
	Text(TextColumnStats),
}

/// This struct contains stats for unknown columns.
#[derive(Clone, Debug)]
pub struct UnknownColumnStats {
	/// This is the name of the column.
	pub column_name: String,
	pub count: usize,
	pub invalid_count: usize,
}

/// This struct contains stats for number columns.
#[derive(Clone, Debug)]
pub struct NumberColumnStats {
	/// This is the name of the column.
	pub column_name: String,
	/// The total number of values.
	pub count: usize,
	/// The total number of valid values.
	pub valid_count: usize,
	/// This is the total number of invalid values. Invalid values are values that fail to parse as finite f32.
	pub invalid_count: usize,
	/// This stores counts for each unique value.
	pub histogram: BTreeMap<Finite<f32>, usize>,
}

/// This struct contains stats for enum columns.
#[derive(Clone, Debug)]
pub struct EnumColumnStats {
	/// This is the name of the column.
	pub column_name: String,
	/// This is the total number of values.
	pub count: usize,
	/// The enum variants.
	pub options: Vec<String>,
	/// This is the total number of valid values.
	pub valid_count: usize,
	/// This is the total number of invalid values.
	pub invalid_count: usize,
	/// This is the histogram.
	pub histogram: Vec<usize>,
}

/// This struct contains stats for text columns.
#[derive(Clone, Debug)]
pub struct TextColumnStats {
	/// This is the name of the column.
	pub column_name: String,
	/// The total number of values.
	pub count: usize,
	/// A map from unigram tokens to the total number of occurrences across all examples.
	pub unigram_histogram: BTreeMap<String, usize>,
	/// A map from bigram tokens to the total number of occurrences across all examples.
	pub bigram_histogram: BTreeMap<String, usize>,
	/// A map from ngrams to the number of examples with at least one occurrence.
	pub per_example_histogram: BTreeMap<String, usize>,
}

/// This struct contains settings used to compute stats.
#[derive(Clone, Debug, PartialEq)]
pub struct StatsSettings {
	/// This is the maximum number of tokens to store in the histogram.
	pub text_histogram_max_size: usize,
	/// This is the maximum number of unique numeric values to store in the histogram.
	pub number_histogram_max_size: usize,
	/// This is the maximum number of tokens to track for text columns.
	pub top_tokens_count: usize,
}

impl Default for StatsSettings {
	fn default() -> Self {
		Self {
			text_histogram_max_size: 100,
			number_histogram_max_size: 100,
			top_tokens_count: 20_000,
		}
	}
}

pub struct StatsOutput(pub Vec<ColumnStatsOutput>);

/// This enum describes the different types of column stats.
#[derive(Debug)]
pub enum ColumnStatsOutput {
	Unknown(UnknownColumnStatsOutput),
	Number(NumberColumnStatsOutput),
	Enum(EnumColumnStatsOutput),
	Text(TextColumnStatsOutput),
}

/// This struct contains stats for unknown columns.
#[derive(Debug)]
pub struct UnknownColumnStatsOutput {
	/// This is the name of the column as it appears in the csv.
	pub column_name: String,
	/// This is the total number of examples that these stats were computed on.
	pub count: u64,
}

/// This struct contains stats for number columns.
#[derive(Debug)]
pub struct NumberColumnStatsOutput {
	/// This is the name of the column as it appears in the csv.
	pub column_name: String,
	/// This is the total number of examples that these stats were computed on.
	pub count: u64,
	/// This is a histogram mapping unique values to their counts. It is `None` if the number of unique values exceeds [`number_histogram_max_size`](struct.StatsSettings.html#structfield.number_histogram_max_size).
	pub histogram: Option<Vec<(f32, u64)>>,
	/// This is the total number of unique values.
	pub unique_count: u64,
	/// This is the max of the values in the column.
	pub max: f32,
	/// This is the mean of the values in the column.
	pub mean: f32,
	/// This is the min of the values in the column.
	pub min: f32,
	/// This is the total number of invalid values. Invalid values are values that fail to parse as floating point numbers.
	pub invalid_count: u64,
	/// This is the variance of the values in the column.
	pub variance: f32,
	/// This is the standard deviation of the values in the column. It is equal to the square root of the variance.
	pub std: f32,
	/// This is the p25, or 25th-percentile value in the column.
	pub p25: f32,
	/// This is the p50, or 50th-percentile value in the column, i.e. the median.
	pub p50: f32,
	/// This is the p75, or 75th-percentile value in the column.
	pub p75: f32,
}

/// This struct contains stats for enum columns.
#[derive(Debug)]
pub struct EnumColumnStatsOutput {
	/// This is the name of the column as it appears in the csv.
	pub column_name: String,
	/// This is the total number of examples that these stats were computed on.
	pub count: u64,
	/// This is a histogram mapping unique variants of the enum to the total count of occurrences of the variant in the dataset.
	pub histogram: Vec<(String, usize)>,
	/// This is the total number of values in the dataset that are invalid. A value is invalid if it is not one of the enum's variants.
	pub invalid_count: usize,
	/// This is the total number of unique values, excluding invalid values.
	pub unique_count: usize,
}

/// This struct contains stats for text columns.
#[derive(Debug)]
pub struct TextColumnStatsOutput {
	/// This is the name of the column as it appears in the csv.
	pub column_name: String,
	/// This is the total number of examples that these stats were computed on.
	pub count: u64,
	/// This contains stats for the top [`top_tokens_count`](struct.StatsSettings.html#top_tokens_count) tokens in the column.
	pub top_tokens: Vec<TokenStats>,
}

/// This struct contains stats for individual tokens
#[derive(Debug)]
pub struct TokenStats {
	pub token: String,
	/// This is the total number of occurrences of this token.
	pub count: u64,
	/// This is the total number of examples that contain this token.
	pub examples_count: u64,
	/// This is the inverse document frequency. [Learn more](https://en.wikipedia.org/wiki/Tf%E2%80%93idf).
	pub idf: f32,
}

impl Stats {
	pub fn compute(dataframe: &DataFrameView, settings: &StatsSettings) -> Self {
		let column_stats = dataframe
			.columns
			.iter()
			.map(|column| ColumnStats::compute(column.view(), &settings))
			.collect();
		Self(column_stats)
	}

	pub fn merge(self, other: Stats) -> Self {
		let column_stats: Vec<ColumnStats> = self
			.0
			.into_iter()
			.zip(other.0.into_iter())
			.map(|(a, b)| a.merge(b))
			.collect();
		Self(column_stats)
	}

	pub fn finalize(self, settings: &StatsSettings) -> StatsOutput {
		let column_stats = self
			.0
			.into_iter()
			.map(|column_stats| column_stats.finalize(settings))
			.collect();
		StatsOutput(column_stats)
	}
}

impl ColumnStats {
	fn compute(column: DataFrameColumnView, settings: &StatsSettings) -> Self {
		match column {
			DataFrameColumnView::Unknown(column) => Self::Unknown(UnknownColumnStats {
				column_name: column.name.to_owned(),
				count: column.len,
				invalid_count: column.len,
			}),
			DataFrameColumnView::Number(column) => {
				Self::Number(NumberColumnStats::compute(column.view(), settings))
			}
			DataFrameColumnView::Enum(column) => {
				Self::Enum(EnumColumnStats::compute(column, settings))
			}
			DataFrameColumnView::Text(column) => {
				Self::Text(TextColumnStats::compute(column, settings))
			}
		}
	}

	fn merge(self, other: Self) -> Self {
		match (self, other) {
			(Self::Unknown(a), Self::Unknown(b)) => Self::Unknown(UnknownColumnStats {
				column_name: a.column_name.to_owned(),
				count: a.count + b.count,
				invalid_count: a.invalid_count + b.invalid_count,
			}),
			(Self::Number(a), Self::Number(b)) => Self::Number(a.merge(b)),
			(Self::Enum(a), Self::Enum(b)) => Self::Enum(a.merge(b)),
			(Self::Text(a), Self::Text(b)) => Self::Text(a.merge(b)),
			_ => unreachable!(),
		}
	}

	fn finalize(self, settings: &StatsSettings) -> ColumnStatsOutput {
		match self {
			Self::Unknown(s) => ColumnStatsOutput::Unknown(stats::UnknownColumnStatsOutput {
				column_name: s.column_name,
				count: s.count.to_u64().unwrap(),
			}),
			Self::Number(s) => ColumnStatsOutput::Number(s.finalize(settings)),
			Self::Enum(s) => ColumnStatsOutput::Enum(s.finalize(settings)),
			Self::Text(s) => ColumnStatsOutput::Text(s.finalize(settings)),
		}
	}
}

impl NumberColumnStats {
	fn compute(column: NumberDataFrameColumnView, _settings: &StatsSettings) -> Self {
		let mut stats = Self {
			column_name: column.name.to_owned(),
			count: column.data.len(),
			histogram: BTreeMap::new(),
			invalid_count: 0,
			valid_count: 0,
		};
		for value in column.data {
			// If the value parses as a finite f32, add it to the histogram. Otherwise, increment the invalid count.
			if let Ok(value) = <Finite<f32>>::new(*value) {
				*stats.histogram.entry(value).or_insert(0) += 1;
				stats.valid_count += 1;
			} else {
				stats.invalid_count += 1;
			}
		}
		stats
	}

	fn merge(mut self, other: Self) -> Self {
		for (value, count) in other.histogram.iter() {
			*self.histogram.entry(*value).or_insert(0) += count;
		}
		self.count += other.count;
		self.invalid_count += other.invalid_count;
		self.valid_count += other.valid_count;
		self
	}

	fn finalize(self, settings: &StatsSettings) -> NumberColumnStatsOutput {
		let unique_values_count = self.histogram.len().to_u64().unwrap();
		let invalid_count = self.invalid_count.to_u64().unwrap();
		let histogram = if self.histogram.len() <= settings.number_histogram_max_size {
			Some(
				self.histogram
					.iter()
					.map(|(value, count)| (value.get(), count.to_u64().unwrap()))
					.collect(),
			)
		} else {
			None
		};
		let min = self.histogram.iter().next().unwrap().0.get();
		let max = self.histogram.iter().next_back().unwrap().0.get();
		let total_values_count = self.valid_count.to_f32().unwrap();
		let quantiles: Vec<f32> = vec![0.25, 0.50, 0.75];
		// Find the index of each quantile given the total number of values in the dataset.
		let quantile_indexes: Vec<usize> = quantiles
			.iter()
			.map(|q| ((total_values_count - 1.0) * q).trunc().to_usize().unwrap())
			.collect();
		// This is the fractiononal part of the index used to interpolate values if the index is not an integer value.
		let quantile_fracts: Vec<f32> = quantiles
			.iter()
			.map(|q| ((total_values_count - 1.0) * q).fract())
			.collect();
		let mut quantiles: Vec<Option<f32>> = vec![None; quantiles.len()];
		let mut current_count: usize = 0;
		let mut mean = 0.0;
		let mut m2 = 0.0;
		let mut iter = self.histogram.iter().peekable();
		while let Some((value, count)) = iter.next() {
			let value = value.get();
			let (new_mean, new_m2) = metrics::merge_mean_m2(
				current_count.to_u64().unwrap(),
				mean,
				m2,
				count.to_u64().unwrap(),
				value.to_f64().unwrap(),
				0.0,
			);
			mean = new_mean;
			m2 = new_m2;
			current_count += count;
			let quantiles_iter = quantiles
				.iter_mut()
				.zip(quantile_indexes.iter().zip(quantile_fracts.iter()))
				.filter(|(q, (_, _))| q.is_none());
			for (quantile, (index, fract)) in quantiles_iter {
				match (current_count - 1).cmp(index) {
					Ordering::Equal => {
						if *fract > 0.0 {
							// Interpolate between two values.
							let next_value = iter.peek().unwrap().0.get();
							*quantile = Some(value * (1.0 - fract) + next_value * fract);
						} else {
							*quantile = Some(value);
						}
					}
					Ordering::Greater => *quantile = Some(value),
					Ordering::Less => {}
				}
			}
		}
		let quantiles: Vec<f32> = quantiles.into_iter().map(|q| q.unwrap()).collect();
		let p25 = quantiles[0];
		let p50 = quantiles[1];
		let p75 = quantiles[2];
		let mean = mean.to_f32().unwrap();
		let variance = metrics::m2_to_variance(
			m2,
			NonZeroU64::new(current_count.to_u64().unwrap()).unwrap(),
		);
		NumberColumnStatsOutput {
			column_name: self.column_name,
			count: self.count.to_u64().unwrap(),
			histogram,
			unique_count: unique_values_count,
			max,
			mean,
			min,
			invalid_count,
			variance,
			std: variance.sqrt(),
			p25,
			p50,
			p75,
		}
	}
}

impl EnumColumnStats {
	fn compute(column: EnumDataFrameColumnView, _settings: &StatsSettings) -> Self {
		let mut histogram = vec![0; column.options.len() + 1];
		for value in column.data {
			let index = value.map(|v| v.get()).unwrap_or(0);
			histogram[index] += 1;
		}
		let invalid_count = histogram[0];
		Self {
			column_name: column.name.to_owned(),
			count: column.data.len(),
			options: column.options.keys().cloned().collect(),
			histogram,
			invalid_count,
			valid_count: 0,
		}
	}

	fn merge(mut self, other: Self) -> Self {
		for (a, b) in self.histogram.iter_mut().zip(other.histogram.iter()) {
			*a += b;
		}
		self.count += other.count;
		self.invalid_count += other.invalid_count;
		self.valid_count += other.valid_count;
		self
	}

	fn finalize(self, _settings: &StatsSettings) -> EnumColumnStatsOutput {
		stats::EnumColumnStatsOutput {
			column_name: self.column_name,
			count: self.count.to_u64().unwrap(),
			invalid_count: self.invalid_count,
			unique_count: self.options.len(),
			histogram: self
				.options
				.into_iter()
				.zip(self.histogram.into_iter().skip(1))
				.map(|(value, count)| (value, count))
				.collect(),
		}
	}
}

impl TextColumnStats {
	fn compute(column: TextDataFrameColumnView, _settings: &StatsSettings) -> Self {
		let mut stats = Self {
			column_name: column.name.to_owned(),
			count: column.data.len(),
			unigram_histogram: BTreeMap::new(),
			bigram_histogram: BTreeMap::new(),
			per_example_histogram: BTreeMap::new(),
		};
		for value in column.data {
			let mut token_set = BTreeSet::new();
			for token in AlphanumericTokenizer::new(value) {
				let token = token.to_string();
				token_set.insert(token.clone());
				*stats.unigram_histogram.entry(token).or_insert(0) += 1;
			}
			for token in token_set.into_iter() {
				*stats.per_example_histogram.entry(token).or_insert(0) += 1;
			}
		}
		stats
	}

	fn merge(mut self, other: Self) -> Self {
		self.count += other.count;
		for (value, count) in other.unigram_histogram.into_iter() {
			if let Some(entry) = self.unigram_histogram.get_mut(&value) {
				*entry += count;
			} else {
				self.unigram_histogram.insert(value, count);
			}
		}
		for (value, count) in other.bigram_histogram.into_iter() {
			if let Some(entry) = self.bigram_histogram.get_mut(&value) {
				*entry += count;
			} else {
				self.bigram_histogram.insert(value, count);
			}
		}
		for (value, count) in other.per_example_histogram.into_iter() {
			if let Some(entry) = self.per_example_histogram.get_mut(&value) {
				*entry += count;
			} else {
				self.per_example_histogram.insert(value, count);
			}
		}
		self
	}

	fn finalize(self, settings: &StatsSettings) -> TextColumnStatsOutput {
		#[derive(Clone, Debug, Eq)]
		struct TokenEntry(String, u64);
		impl std::cmp::Ord for TokenEntry {
			fn cmp(&self, other: &Self) -> std::cmp::Ordering {
				self.1.cmp(&other.1)
			}
		}
		impl std::cmp::PartialOrd for TokenEntry {
			fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
				self.1.partial_cmp(&other.1)
			}
		}
		impl std::cmp::PartialEq for TokenEntry {
			fn eq(&self, other: &Self) -> bool {
				self.1.eq(&other.1)
			}
		}
		let mut top_tokens = std::collections::BinaryHeap::new();
		for (token, count) in self.unigram_histogram.iter() {
			top_tokens.push(TokenEntry(token.clone(), count.to_u64().unwrap()));
		}
		for (token, count) in self.bigram_histogram.iter() {
			top_tokens.push(TokenEntry(token.clone(), count.to_u64().unwrap()));
		}
		let n_examples = self.count.to_u64().unwrap();
		let top_tokens = (0..settings.top_tokens_count)
			.map(|_| top_tokens.pop())
			.filter_map(|token_entry| token_entry.map(|token_entry| (token_entry.0, token_entry.1)))
			.map(|(token, count)| {
				let examples_count = self
					.per_example_histogram
					.get(&token)
					.unwrap()
					.to_u64()
					.unwrap();
				// This is the "inverse document frequency smooth" form of the IDF. [Learn more](https://en.wikipedia.org/wiki/Tf%E2%80%93idf).
				let idf = (n_examples.to_f32().unwrap() / (1.0 + examples_count.to_f32().unwrap()))
					.ln() + 1.0;
				TokenStats {
					token,
					count,
					examples_count,
					idf,
				}
			})
			.collect::<Vec<TokenStats>>();
		TextColumnStatsOutput {
			column_name: self.column_name,
			count: self.count.to_u64().unwrap(),
			top_tokens,
		}
	}
}

impl ColumnStatsOutput {
	/// Return the name of the source column.
	pub fn column_name(&self) -> &str {
		match self {
			Self::Unknown(value) => &value.column_name,
			Self::Number(value) => &value.column_name,
			Self::Enum(value) => &value.column_name,
			Self::Text(value) => &value.column_name,
		}
	}
}
