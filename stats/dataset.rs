use super::StatsSettings;
use crate::{dataframe::*, util::finite::Finite, util::text};
use std::collections::{BTreeMap, BTreeSet};

// TODO use max histogram size from stats settings, it's ignored right now.
#[derive(Clone, Debug)]
pub enum DatasetStats {
	Unknown(UnknownDatasetStats),
	Number(NumberDatasetStats),
	Enum(EnumDatasetStats),
	Text(TextDatasetStats),
}

#[derive(Clone, Debug)]
pub struct UnknownDatasetStats {
	pub count: usize,
	pub invalid_count: usize,
}

#[derive(Clone, Debug)]
pub struct NumberDatasetStats {
	pub count: usize,
	pub valid_count: usize,
	pub invalid_count: usize,
	pub histogram: BTreeMap<Finite<f32>, usize>,
}

#[derive(Clone, Debug)]
pub struct EnumDatasetStats {
	pub count: usize,
	pub options: Vec<String>,
	pub valid_count: usize,
	pub invalid_count: usize,
	pub histogram: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct TextDatasetStats {
	pub count: usize,
	pub tokenizer: text::AlphanumericTokenizer,
	/// A map from unigram tokens to the total number of occurrences across all examples
	pub unigram_histogram: BTreeMap<String, usize>,
	/// A map from bigram tokens to the total number of occurrences across all examples
	pub bigram_histogram: BTreeMap<String, usize>,
	/// A map from ngrams to the number of examples with at least one occurrence
	pub per_example_histogram: BTreeMap<String, usize>,
}

impl DatasetStats {
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
		stats
	}
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
