use crate::stats;
use fnv::{FnvHashMap, FnvHashSet};
use itertools::Itertools;
use num_traits::ToPrimitive;
use std::{cmp::Ordering, collections::BTreeMap, num::NonZeroU64};
use tangram_dataframe::prelude::*;
use tangram_metrics as metrics;
use tangram_util::{alphanumeric_tokenizer::AlphanumericTokenizer, finite::Finite, zip};

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
	/// A map from tokens to the total number of occurrences of that token across all examples.
	pub token_occurrence_histogram: FnvHashMap<Token, usize>,
	/// A map from token to the number of examples with at least one occurrence.
	pub token_example_histogram: FnvHashMap<Token, usize>,
	pub tokenizer: Tokenizer,
}

#[derive(Clone, Debug)]
pub enum Tokenizer {
	Alphanumeric,
}

/// This struct contains settings used to compute stats.
#[derive(Clone, Debug, PartialEq)]
pub struct StatsSettings {
	/// This is the maximum number of tokens to store in the histogram.
	pub token_histogram_max_size: usize,
	/// This is the maximum number of unique numeric values to store in the histogram.
	pub number_histogram_max_size: usize,
	/// This is the maximum number of tokens to track for text columns.
	pub top_tokens_count: usize,
}

impl Default for StatsSettings {
	fn default() -> StatsSettings {
		StatsSettings {
			token_histogram_max_size: 100,
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

impl ColumnStatsOutput {
	/// Return the name of the source column.
	pub fn column_name(&self) -> &str {
		match self {
			ColumnStatsOutput::Unknown(value) => &value.column_name,
			ColumnStatsOutput::Number(value) => &value.column_name,
			ColumnStatsOutput::Enum(value) => &value.column_name,
			ColumnStatsOutput::Text(value) => &value.column_name,
		}
	}
}

/// This struct contains stats for unknown columns.
#[derive(Debug)]
pub struct UnknownColumnStatsOutput {
	/// This is the name of the column as it appears in the csv.
	pub column_name: String,
	/// This is the total number of examples that these stats were computed on.
	pub count: usize,
}

/// This struct contains stats for number columns.
#[derive(Debug)]
pub struct NumberColumnStatsOutput {
	/// This is the name of the column as it appears in the csv.
	pub column_name: String,
	/// This is the total number of examples that these stats were computed on.
	pub count: usize,
	/// This is a histogram mapping unique values to their counts. It is `None` if the number of unique values exceeds [`number_histogram_max_size`](struct.StatsSettings.html#structfield.number_histogram_max_size).
	pub histogram: Option<Vec<(Finite<f32>, usize)>>,
	/// This is the total number of unique values.
	pub unique_count: usize,
	/// This is the max of the values in the column.
	pub max: f32,
	/// This is the mean of the values in the column.
	pub mean: f32,
	/// This is the min of the values in the column.
	pub min: f32,
	/// This is the total number of invalid values. Invalid values are values that fail to parse as floating point numbers.
	pub invalid_count: usize,
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
	// This enum is used to determine the method to split text into tokens.
	pub tokenizer: Tokenizer,
}

impl Stats {
	pub fn compute(dataframe: &DataFrameView, settings: &StatsSettings) -> Stats {
		let column_stats = dataframe
			.columns()
			.iter()
			.map(|column| ColumnStats::compute(column.view(), &settings))
			.collect();
		Stats(column_stats)
	}

	pub fn merge(self, other: Stats) -> Stats {
		let column_stats: Vec<ColumnStats> =
			zip!(self.0, other.0).map(|(a, b)| a.merge(b)).collect();
		Stats(column_stats)
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
	fn compute(column: DataFrameColumnView, settings: &StatsSettings) -> ColumnStats {
		match column {
			DataFrameColumnView::Unknown(column) => ColumnStats::Unknown(UnknownColumnStats {
				column_name: column.name().unwrap().to_owned(),
				count: column.len(),
				invalid_count: column.len(),
			}),
			DataFrameColumnView::Number(column) => {
				ColumnStats::Number(NumberColumnStats::compute(column.view(), settings))
			}
			DataFrameColumnView::Enum(column) => {
				ColumnStats::Enum(EnumColumnStats::compute(column, settings))
			}
			DataFrameColumnView::Text(column) => {
				ColumnStats::Text(TextColumnStats::compute(column, settings))
			}
		}
	}

	fn merge(self, other: ColumnStats) -> ColumnStats {
		match (self, other) {
			(ColumnStats::Unknown(a), ColumnStats::Unknown(b)) => {
				ColumnStats::Unknown(UnknownColumnStats {
					column_name: a.column_name.clone(),
					count: a.count + b.count,
					invalid_count: a.invalid_count + b.invalid_count,
				})
			}
			(ColumnStats::Number(a), ColumnStats::Number(b)) => ColumnStats::Number(a.merge(b)),
			(ColumnStats::Enum(a), ColumnStats::Enum(b)) => ColumnStats::Enum(a.merge(b)),
			(ColumnStats::Text(a), ColumnStats::Text(b)) => ColumnStats::Text(a.merge(b)),
			_ => unreachable!(),
		}
	}

	fn finalize(self, settings: &StatsSettings) -> ColumnStatsOutput {
		match self {
			ColumnStats::Unknown(column_stats_output) => {
				ColumnStatsOutput::Unknown(stats::UnknownColumnStatsOutput {
					column_name: column_stats_output.column_name,
					count: column_stats_output.count,
				})
			}
			ColumnStats::Number(column_stats_output) => {
				ColumnStatsOutput::Number(column_stats_output.finalize(settings))
			}
			ColumnStats::Enum(column_stats_output) => {
				ColumnStatsOutput::Enum(column_stats_output.finalize(settings))
			}
			ColumnStats::Text(column_stats_output) => {
				ColumnStatsOutput::Text(column_stats_output.finalize(settings))
			}
		}
	}
}

impl NumberColumnStats {
	fn compute(column: NumberDataFrameColumnView, _settings: &StatsSettings) -> NumberColumnStats {
		let mut stats = NumberColumnStats {
			column_name: column.name().unwrap().to_owned(),
			count: column.len(),
			histogram: BTreeMap::new(),
			invalid_count: 0,
			valid_count: 0,
		};
		for value in column.iter() {
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

	fn merge(mut self, other: NumberColumnStats) -> NumberColumnStats {
		for (value, count) in other.histogram.iter() {
			*self.histogram.entry(*value).or_insert(0) += count;
		}
		self.count += other.count;
		self.invalid_count += other.invalid_count;
		self.valid_count += other.valid_count;
		self
	}

	fn finalize(self, settings: &StatsSettings) -> NumberColumnStatsOutput {
		let unique_values_count = self.histogram.len();
		let invalid_count = self.invalid_count;
		let histogram = if self.histogram.len() <= settings.number_histogram_max_size {
			Some(self.histogram.iter().map(|(k, v)| (*k, *v)).collect())
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
				value as f64,
				0.0,
			);
			mean = new_mean;
			m2 = new_m2;
			current_count += count;
			let quantiles_iter = zip!(
				quantiles.iter_mut(),
				quantile_indexes.iter(),
				quantile_fracts.iter(),
			)
			.filter(|(quantile, _, _)| quantile.is_none());
			for (quantile, index, fract) in quantiles_iter {
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
			count: self.count,
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
	fn compute(column: EnumDataFrameColumnView, _settings: &StatsSettings) -> EnumColumnStats {
		let mut histogram = vec![0; column.options().len() + 1];
		for value in column.iter() {
			let index = value.map(|v| v.get()).unwrap_or(0);
			histogram[index] += 1;
		}
		let invalid_count = histogram[0];
		EnumColumnStats {
			column_name: column.name().unwrap().to_owned(),
			count: column.len(),
			options: column.options().to_owned(),
			histogram,
			invalid_count,
			valid_count: 0,
		}
	}

	fn merge(mut self, other: EnumColumnStats) -> EnumColumnStats {
		for (a, b) in zip!(self.histogram.iter_mut(), other.histogram.iter()) {
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
			histogram: zip!(self.options, self.histogram.into_iter().skip(1))
				.map(|(value, count)| (value, count))
				.collect(),
		}
	}
}

/// This struct contains stats for individual tokens
#[derive(Debug)]
pub struct TokenStats {
	pub token: Token,
	/// This is the number of occurrences of this token across all examples.
	pub count: usize,
	/// This is the number of examples that contain at least one occurrence of this token.
	pub examples_count: usize,
	/// This is the inverse document frequency. [Learn more](https://en.wikipedia.org/wiki/Tf%E2%80%93idf).
	pub idf: f32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
	Unigram(String),
	Bigram(String, String),
}

#[derive(Clone, Debug, Eq)]
struct TokenEntry(pub Token, pub usize);
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

impl TextColumnStats {
	fn compute(column: TextDataFrameColumnView, _settings: &StatsSettings) -> TextColumnStats {
		let mut stats = TextColumnStats {
			column_name: column.name().unwrap().to_owned(),
			count: column.len(),
			token_occurrence_histogram: FnvHashMap::default(),
			token_example_histogram: FnvHashMap::default(),
			tokenizer: Tokenizer::Alphanumeric,
		};
		for value in column.iter() {
			let mut token_set = FnvHashSet::default();
			for unigram in AlphanumericTokenizer::new(value) {
				let unigram = Token::Unigram(unigram.into_owned());
				token_set.insert(unigram.clone());
				*stats.token_occurrence_histogram.entry(unigram).or_insert(0) += 1;
			}
			for (token_a, token_b) in AlphanumericTokenizer::new(value).tuple_windows() {
				let bigram = Token::Bigram(token_a.into_owned(), token_b.into_owned());
				token_set.insert(bigram.clone());
				*stats.token_occurrence_histogram.entry(bigram).or_insert(0) += 1;
			}
			for token in token_set.into_iter() {
				*stats.token_example_histogram.entry(token).or_insert(0) += 1;
			}
		}
		stats
	}

	fn merge(mut self, other: TextColumnStats) -> TextColumnStats {
		self.count += other.count;
		for (value, count) in other.token_occurrence_histogram.into_iter() {
			if let Some(entry) = self.token_occurrence_histogram.get_mut(&value) {
				*entry += count;
			} else {
				self.token_occurrence_histogram.insert(value, count);
			}
		}
		for (value, count) in other.token_example_histogram.into_iter() {
			if let Some(entry) = self.token_example_histogram.get_mut(&value) {
				*entry += count;
			} else {
				self.token_example_histogram.insert(value, count);
			}
		}
		self
	}

	fn finalize(self, settings: &StatsSettings) -> TextColumnStatsOutput {
		let mut top_tokens = std::collections::BinaryHeap::new();
		for (token, count) in self.token_occurrence_histogram.iter() {
			top_tokens.push(TokenEntry(token.clone(), *count));
		}
		let n_examples = self.count.to_u64().unwrap();
		let top_tokens = (0..settings.top_tokens_count)
			.map(|_| top_tokens.pop())
			.filter_map(|token_entry| token_entry.map(|token_entry| (token_entry.0, token_entry.1)))
			.map(|(token, count)| {
				let examples_count = self.token_example_histogram[&token];
				let idf = ((1.0 + n_examples.to_f32().unwrap())
					/ (1.0 + examples_count.to_f32().unwrap()))
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
			tokenizer: Tokenizer::Alphanumeric,
		}
	}
}
