use super::StatsSettings;
use crate::{dataframe::*, util::finite::Finite, util::text};
use std::collections::{BTreeMap, BTreeSet};

// This is an enum describing the different types of stats where the type matches the type of the source column.
#[derive(Clone, Debug)]
pub enum DatasetStats {
	Unknown(UnknownDatasetStats),
	Number(NumberDatasetStats),
	Enum(EnumDatasetStats),
	Text(TextDatasetStats),
}

/// This struct contains stats for unknown columns.
#[derive(Clone, Debug)]
pub struct UnknownDatasetStats {
	pub count: usize,
	pub invalid_count: usize,
}

/// This struct contains stats for number columns.
#[derive(Clone, Debug)]
pub struct NumberDatasetStats {
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
pub struct EnumDatasetStats {
	/// The total number of values.
	pub count: usize,
	/// The enum variants.
	pub options: Vec<String>,
	/// The total number of valid values.
	pub valid_count: usize,
	/// The total number of invalid values.
	pub invalid_count: usize,
	/// Stores counts for each enum variant.
	/// The i-th entry in the vec corresponds to the count for the i-th enum variant in the options.
	pub histogram: Vec<usize>,
}

/// This struct contains stats for text columns.
#[derive(Clone, Debug)]
pub struct TextDatasetStats {
	/// The total number of values.
	pub count: usize,
	/// The tokenizer is used to split text into tokens.
	pub tokenizer: text::AlphanumericTokenizer,
	/// A map from unigram tokens to the total number of occurrences across all examples.
	pub unigram_histogram: BTreeMap<String, usize>,
	/// A map from bigram tokens to the total number of occurrences across all examples.
	pub bigram_histogram: BTreeMap<String, usize>,
	/// A map from ngrams to the number of examples with at least one occurrence.
	pub per_example_histogram: BTreeMap<String, usize>,
}

impl DatasetStats {
	/// Compute the stats for a given column and settings.
	pub fn compute(column: &ColumnView, settings: &StatsSettings) -> Self {
		match column {
			ColumnView::Unknown(column) => Self::Unknown(UnknownDatasetStats {
				count: column.len,
				invalid_count: column.len,
			}),
			ColumnView::Number(column) => {
				Self::Number(NumberDatasetStats::compute(column, settings))
			}
			ColumnView::Enum(column) => Self::Enum(EnumDatasetStats::compute(column, settings)),
			ColumnView::Text(column) => Self::Text(TextDatasetStats::compute(column, settings)),
		}
	}

	/// Merge two stats structs of the same type together. This is useful for parallel computation of stats.
	pub fn merge(&self, other: &Self) -> Self {
		match (self, other) {
			(Self::Unknown(a), Self::Unknown(b)) => Self::Unknown(UnknownDatasetStats {
				count: a.count + b.count,
				invalid_count: a.invalid_count + b.invalid_count,
			}),
			(Self::Number(a), Self::Number(b)) => Self::Number(a.merge(b)),
			(Self::Enum(a), Self::Enum(b)) => Self::Enum(a.merge(b)),
			(Self::Text(a), Self::Text(b)) => Self::Text(a.merge(b)),
			_ => unreachable!(),
		}
	}
}

impl NumberDatasetStats {
	/// Compute the stats for a number column.
	pub fn compute(column: &NumberColumnView, _settings: &StatsSettings) -> Self {
		let mut stats = Self {
			count: column.data.len(),
			histogram: BTreeMap::new(),
			invalid_count: 0,
			valid_count: 0,
		};
		for value in column.data {
			// if value parses as finite f32, add it to the histogram
			// otherwise, increment the invalid count
			if let Ok(value) = <Finite<f32>>::new(*value) {
				*stats.histogram.entry(value).or_insert(0) += 1;
				stats.valid_count += 1;
			} else {
				stats.invalid_count += 1;
			}
		}
		stats
	}
	/// Merge two number stats structs together. This is useful for parallel computation of stats.
	pub fn merge(&self, other: &Self) -> Self {
		let mut stats = self.clone();
		for (value, count) in other.histogram.iter() {
			*stats.histogram.entry(*value).or_insert(0) += count;
		}
		stats.count += other.count;
		stats.invalid_count += other.invalid_count;
		stats.valid_count += other.valid_count;
		stats
	}
}

impl EnumDatasetStats {
	/// Compute the stats for an enum column.
	pub fn compute(column: &EnumColumnView, _settings: &StatsSettings) -> Self {
		let mut stats = Self {
			count: column.data.len(),
			options: column.options.to_owned(),
			histogram: vec![0; column.options.len() + 1],
			invalid_count: 0,
			valid_count: 0,
		};
		for value in column.data {
			stats.histogram[*value] += 1;
		}
		stats.invalid_count = stats.histogram[0];
		stats
	}
	/// Merge two enum stats structs together. This is useful for parallel computation of stats.
	pub fn merge(&self, other: &Self) -> Self {
		let mut stats = self.clone();
		for (a, b) in stats.histogram.iter_mut().zip(other.histogram.iter()) {
			*a += b;
		}
		stats.count += other.count;
		stats.invalid_count += other.invalid_count;
		stats.valid_count += other.valid_count;
		stats
	}
}

impl TextDatasetStats {
	/// Compute the stats for a text column.
	pub fn compute(column: &TextColumnView, _settings: &StatsSettings) -> Self {
		let mut stats = Self {
			count: column.data.len(),
			tokenizer: text::AlphanumericTokenizer {},
			unigram_histogram: BTreeMap::new(),
			bigram_histogram: BTreeMap::new(),
			per_example_histogram: BTreeMap::new(),
		};
		for value in column.data {
			let mut token_set = BTreeSet::new();
			let tokens = stats.tokenizer.tokenize(value);
			let bigrams = text::bigrams(&tokens);
			for token in tokens.into_iter() {
				token_set.insert(token.clone());
				*stats.unigram_histogram.entry(token).or_insert(0) += 1;
			}
			for bigram in bigrams.into_iter() {
				token_set.insert(bigram.clone());
				*stats.bigram_histogram.entry(bigram).or_insert(0) += 1;
			}
			for token in token_set.into_iter() {
				*stats.per_example_histogram.entry(token).or_insert(0) += 1;
			}
		}
		stats
	}
	/// Merge two text stats structs together. This is useful for parallel computation of stats.
	pub fn merge(&self, other: &Self) -> Self {
		let mut stats = self.clone();
		stats.count += other.count;
		for (value, count) in other.unigram_histogram.iter() {
			if let Some(entry) = stats.unigram_histogram.get_mut(value) {
				*entry += count;
			} else {
				stats.unigram_histogram.insert(value.clone(), *count);
			}
		}
		for (value, count) in other.bigram_histogram.iter() {
			if let Some(entry) = stats.bigram_histogram.get_mut(value) {
				*entry += count;
			} else {
				stats.bigram_histogram.insert(value.clone(), *count);
			}
		}
		for (value, count) in other.per_example_histogram.iter() {
			if let Some(entry) = stats.per_example_histogram.get_mut(value) {
				*entry += count;
			} else {
				stats.per_example_histogram.insert(value.clone(), *count);
			}
		}
		stats
	}
}
