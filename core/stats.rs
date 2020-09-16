use crate::text;
use crate::{stats, train::StatsProgress};
use num_traits::ToPrimitive;
use std::{
	cmp::Ordering,
	collections::{BTreeMap, BTreeSet},
	num::NonZeroU64,
};
use tangram_dataframe::*;
use tangram_finite::Finite;
use tangram_metrics as metrics;
use tangram_progress::ProgressCounter;

/// This struct contains settings used to compute stats.
#[derive(Clone, Debug, PartialEq)]
pub struct StatsSettings {
	/// This is the maximum number of tokens to store in the histogram.
	pub text_histogram_max_size: usize,
	/// This is the maximum number of unique numeric values to store in the histogram.
	pub number_histogram_max_size: usize,
	/// This is the maximum number of tokens to track for text columns.
	pub top_tokens_count: usize,
	/// This is the minimum number of rows with a token for it to be considered for tracking.
	pub min_document_frequency: usize,
}

impl Default for StatsSettings {
	fn default() -> Self {
		Self {
			text_histogram_max_size: 100,
			number_histogram_max_size: 100,
			top_tokens_count: 20_000,
			min_document_frequency: 2,
		}
	}
}

/// This struct is the output from `compute_stats`. It contains stats for the overall dataset and also stats for just the train and test portions.
pub struct ComputeStatsOutput {
	/// This reports column stats for the whole dataset.
	pub overall_column_stats: Vec<ColumnStatsOutput>,
	/// This reports stats for the train portion of the dataset.
	pub train_column_stats: Vec<ColumnStatsOutput>,
	/// This reports stats for the test portion of the dataset.
	pub test_column_stats: Vec<ColumnStatsOutput>,
}

/// An enum describing the different types of column stats.
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
	/// This is a list of the most frequently occurring tokens, as well as the number of occurrences in the dataset and its IDF score.
	pub top_tokens: Vec<(String, u64, f32)>,
}

/// This function computes a range of statistics on the data in the train and test dataframes provided. After some transformations, these become the stats that appear under the "Training Stats" section in the reporting and monitoring web app.
pub fn compute_stats(
	dataframe_train: &DataFrameView,
	dataframe_test: &DataFrameView,
	settings: &StatsSettings,
	update_progress: &mut impl FnMut(StatsProgress),
) -> ComputeStatsOutput {
	let n_cols = dataframe_train.ncols();
	let n_rows = dataframe_train.nrows() + dataframe_test.nrows();

	// compute histograms
	// first we collect the whole dataset into histograms
	// then we will use these histograms to compute subsequent statistics
	let dataset_counter = ProgressCounter::new(n_cols.to_u64().unwrap() * n_rows.to_u64().unwrap());
	update_progress(StatsProgress::DatasetStats(dataset_counter));
	let train_dataset_stats: Vec<DatasetStats> = dataframe_train
		.columns
		.iter()
		.map(|column| DatasetStats::compute(column, &settings))
		.collect();
	let test_dataset_stats: Vec<DatasetStats> = dataframe_test
		.columns
		.iter()
		.map(|column| DatasetStats::compute(column, &settings))
		.collect();
	// rather than recompute stats over the whole data
	let overall_dataset_stats: Vec<DatasetStats> = train_dataset_stats
		.iter()
		.cloned()
		.zip(test_dataset_stats.iter().cloned())
		.map(|(a, b)| a.merge(b))
		.collect();

	// compute histogram stats
	let n_histogram_entries_train: usize = train_dataset_stats
		.iter()
		.map(|stats| match stats {
			DatasetStats::Unknown(_) => 0,
			DatasetStats::Number(s) => s.histogram.len(),
			DatasetStats::Enum(_) => 0,
			DatasetStats::Text(_) => 0,
		})
		.sum();
	let n_histogram_entries_test: usize = train_dataset_stats
		.iter()
		.map(|stats| match stats {
			DatasetStats::Unknown(_) => 0,
			DatasetStats::Number(s) => s.histogram.len(),
			DatasetStats::Enum(_) => 0,
			DatasetStats::Text(_) => 0,
		})
		.sum();
	let n_histogram_entries_overall: usize = n_histogram_entries_train + n_histogram_entries_test;
	let n = n_histogram_entries_train + n_histogram_entries_test + n_histogram_entries_overall;
	let n = n.to_u64().unwrap();
	let histogram_counter = ProgressCounter::new(n);
	update_progress(StatsProgress::HistogramStats(histogram_counter.clone()));
	let train_histogram_stats: Vec<HistogramStats> = train_dataset_stats
		.iter()
		.map(|h| compute_histogram_stats(h, || histogram_counter.inc(1)))
		.collect();
	let test_histogram_stats: Vec<HistogramStats> = test_dataset_stats
		.iter()
		.map(|h| compute_histogram_stats(h, || histogram_counter.inc(1)))
		.collect();
	let overall_histogram_stats: Vec<HistogramStats> = overall_dataset_stats
		.iter()
		.map(|h| compute_histogram_stats(h, || histogram_counter.inc(1)))
		.collect();

	// transform histograms and histogram_stats into column_stats
	let train_column_stats =
		compute_column_stats(&train_dataset_stats, train_histogram_stats, &settings);
	let test_column_stats =
		compute_column_stats(&test_dataset_stats, test_histogram_stats, &settings);
	let overall_column_stats =
		compute_column_stats(&overall_dataset_stats, overall_histogram_stats, &settings);

	ComputeStatsOutput {
		overall_column_stats,
		test_column_stats,
		train_column_stats,
	}
}

fn compute_column_stats(
	dataset_stats: &[DatasetStats],
	histogram_stats: Vec<HistogramStats>,
	settings: &StatsSettings,
) -> Vec<stats::ColumnStatsOutput> {
	dataset_stats
		.iter()
		.zip(histogram_stats.into_iter())
		.map(|(dataset_stats, histogram_stats)| {
			compute_column_stats_for_column(dataset_stats, histogram_stats, settings)
		})
		.collect()
}

fn compute_column_stats_for_column(
	dataset_stats: &DatasetStats,
	histogram_stats: HistogramStats,
	settings: &StatsSettings,
) -> stats::ColumnStatsOutput {
	match (dataset_stats, &histogram_stats) {
		(DatasetStats::Unknown(dataset_stats), _) => {
			stats::ColumnStatsOutput::Unknown(stats::UnknownColumnStatsOutput {
				column_name: dataset_stats.column_name.to_owned(),
				count: dataset_stats.count.to_u64().unwrap(),
			})
		}
		(DatasetStats::Text(dataset_stats), _) => {
			compute_column_stats_text(dataset_stats, settings)
		}
		(DatasetStats::Number(dataset_stats), HistogramStats::Number(histogram_stats)) => {
			compute_number_column_stats(dataset_stats, histogram_stats, settings)
		}
		(DatasetStats::Enum(dataset_stats), _) => {
			compute_enum_column_stats(dataset_stats, settings)
		}
		_ => unreachable!(),
	}
}

/// Compute [ColumnStats](struct.ColumnStats.html) for a number column by combining stats computed in dataset_stats and histogram_stats.
fn compute_number_column_stats(
	dataset_stats: &NumberDatasetStats,
	histogram_stats: &NumberHistogramStats,
	settings: &StatsSettings,
) -> stats::ColumnStatsOutput {
	let unique_values_count = dataset_stats.histogram.len().to_u64().unwrap();
	let invalid_count = dataset_stats.invalid_count.to_u64().unwrap();
	let histogram = if dataset_stats.histogram.len() <= settings.number_histogram_max_size {
		Some(
			dataset_stats
				.histogram
				.iter()
				.map(|(value, count)| (value.get(), count.to_u64().unwrap()))
				.collect(),
		)
	} else {
		None
	};
	stats::ColumnStatsOutput::Number(stats::NumberColumnStatsOutput {
		column_name: dataset_stats.column_name.to_owned(),
		count: dataset_stats.count.to_u64().unwrap(),
		histogram,
		unique_count: unique_values_count,
		max: histogram_stats.max,
		mean: histogram_stats.mean,
		min: histogram_stats.min,
		invalid_count,
		variance: histogram_stats.variance,
		std: histogram_stats.variance.sqrt(),
		p25: histogram_stats.p25,
		p50: histogram_stats.p50,
		p75: histogram_stats.p75,
	})
}

/// Compute [ColumnStats](struct.ColumnStats.html) for an enum column.
fn compute_enum_column_stats(
	dataset_stats: &EnumDatasetStats,
	_settings: &StatsSettings,
) -> stats::ColumnStatsOutput {
	stats::ColumnStatsOutput::Enum(stats::EnumColumnStatsOutput {
		column_name: dataset_stats.column_name.to_owned(),
		count: dataset_stats.count.to_u64().unwrap(),
		invalid_count: dataset_stats.invalid_count,
		unique_count: dataset_stats.options.len(),
		histogram: dataset_stats
			.options
			.iter()
			.zip(dataset_stats.histogram.iter().skip(1))
			.map(|(value, count)| ((*value).to_string(), *count))
			.collect(),
	})
}

/// Compute [ColumnStats](struct.ColumnStats.html) for a text column.
fn compute_column_stats_text(
	dataset_stats: &TextDatasetStats,
	settings: &StatsSettings,
) -> stats::ColumnStatsOutput {
	let mut top_tokens = std::collections::BinaryHeap::new();
	for (token, count) in dataset_stats.unigram_histogram.iter() {
		let entry = TokenEntry(token.clone(), count.to_u64().unwrap());
		top_tokens.push(entry);
	}
	for (token, count) in dataset_stats.bigram_histogram.iter() {
		let entry = TokenEntry(token.clone(), count.to_u64().unwrap());
		top_tokens.push(entry);
	}
	let top_tokens = (0..settings.top_tokens_count)
		.map(|_| top_tokens.pop())
		.filter_map(|token_entry| token_entry.map(|token_entry| (token_entry.0, token_entry.1)))
		.filter_map(|(token, count)| {
			let document_frequency = dataset_stats.per_example_histogram.get(&token).unwrap();
			if *document_frequency >= settings.min_document_frequency.to_usize().unwrap() {
				// idf = log ((n + 1) / (1 + document_frequency)) + 1
				let n_examples = dataset_stats.count;
				let idf = ((1.0 + n_examples.to_f32().unwrap())
					/ (1.0 + (document_frequency.to_f32().unwrap())))
				.ln() + 1.0;
				Some((token, count, idf))
			} else {
				None
			}
		})
		.collect::<Vec<(String, u64, f32)>>();
	stats::ColumnStatsOutput::Text(stats::TextColumnStatsOutput {
		column_name: dataset_stats.column_name.to_owned(),
		count: dataset_stats.count.to_u64().unwrap(),
		top_tokens,
	})
}

#[derive(Eq, Debug, Clone)]
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
	/// This is the name of the column.
	pub column_name: String,
	pub count: usize,
	pub invalid_count: usize,
}

/// This struct contains stats for number columns.
#[derive(Clone, Debug)]
pub struct NumberDatasetStats {
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
pub struct EnumDatasetStats {
	/// This is the name of the column.
	pub column_name: String,
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
	/// This is the name of the column.
	pub column_name: String,
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
				column_name: column.name.to_owned(),
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
	pub fn merge(self, other: Self) -> Self {
		match (self, other) {
			(Self::Unknown(a), Self::Unknown(b)) => Self::Unknown(UnknownDatasetStats {
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
}

impl NumberDatasetStats {
	/// Compute the stats for a number column.
	pub fn compute(column: &NumberColumnView, _settings: &StatsSettings) -> Self {
		let mut stats = Self {
			column_name: column.name.to_owned(),
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
	pub fn merge(mut self, other: Self) -> Self {
		for (value, count) in other.histogram.iter() {
			*self.histogram.entry(*value).or_insert(0) += count;
		}
		self.count += other.count;
		self.invalid_count += other.invalid_count;
		self.valid_count += other.valid_count;
		self
	}
}

impl EnumDatasetStats {
	/// Compute the stats for an enum column.
	pub fn compute(column: &EnumColumnView, _settings: &StatsSettings) -> Self {
		let mut stats = Self {
			column_name: column.name.to_owned(),
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
	pub fn merge(mut self, other: Self) -> Self {
		for (a, b) in self.histogram.iter_mut().zip(other.histogram.iter()) {
			*a += b;
		}
		self.count += other.count;
		self.invalid_count += other.invalid_count;
		self.valid_count += other.valid_count;
		self
	}
}

impl TextDatasetStats {
	/// Compute the stats for a text column.
	pub fn compute(column: &TextColumnView, _settings: &StatsSettings) -> Self {
		let mut stats = Self {
			column_name: column.name.to_owned(),
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
	pub fn merge(mut self, other: Self) -> Self {
		self.count += other.count;
		for (value, count) in other.unigram_histogram.iter() {
			if let Some(entry) = self.unigram_histogram.get_mut(value) {
				*entry += count;
			} else {
				self.unigram_histogram.insert(value.clone(), *count);
			}
		}
		for (value, count) in other.bigram_histogram.iter() {
			if let Some(entry) = self.bigram_histogram.get_mut(value) {
				*entry += count;
			} else {
				self.bigram_histogram.insert(value.clone(), *count);
			}
		}
		for (value, count) in other.per_example_histogram.iter() {
			if let Some(entry) = self.per_example_histogram.get_mut(value) {
				*entry += count;
			} else {
				self.per_example_histogram.insert(value.clone(), *count);
			}
		}
		self
	}
}

/// HistogramStats contain statistics computed using aggregated histograms of the original column. We use aggregated histogram statistics for computing quantiles on number columns. Instead of sorting `O(n_examples)`, we only need to sort `O(n_unique_values)`.
#[derive(Debug, PartialEq)]
pub enum HistogramStats {
	Unknown(UnknownHistogramStats),
	Text(TextHistogramStats),
	Number(NumberHistogramStats),
	Enum(EnumHistogramStats),
}

/// UnknownHistogramStats are empty.
#[derive(Debug, PartialEq)]
pub struct UnknownHistogramStats {}

/// TextHistogramStats are empty.
#[derive(Debug, PartialEq)]
pub struct TextHistogramStats {}

/// NumberHistogramStats contain statistics computed using aggregated histograms of the original column.
#[derive(Debug, PartialEq)]
pub struct NumberHistogramStats {
	pub mean: f32,
	pub variance: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
	pub max: f32,
}

/// EnumHistogramStats are emtpy.
#[derive(Debug, PartialEq)]
pub struct EnumHistogramStats {}

/// Compute stats using the `dataset_stats` which contain histograms of the original data.
pub fn compute_histogram_stats(
	dataset_stats: &DatasetStats,
	progress: impl Fn(),
) -> HistogramStats {
	match dataset_stats {
		DatasetStats::Unknown(_) => HistogramStats::Unknown(UnknownHistogramStats {}),
		DatasetStats::Number(dataset_stats) => {
			HistogramStats::Number(compute_number_histogram_stats(
				&dataset_stats.histogram,
				dataset_stats.valid_count,
				progress,
			))
		}
		DatasetStats::Enum(_) => HistogramStats::Enum(EnumHistogramStats {}),
		DatasetStats::Text(_) => HistogramStats::Text(TextHistogramStats {}),
	}
}

/// Compute stats for number columns using the `dataset_stats` which contain histograms of the original data.
fn compute_number_histogram_stats(
	histogram: &BTreeMap<Finite<f32>, usize>,
	total_values_count: usize,
	progress: impl Fn(),
) -> NumberHistogramStats {
	let min = histogram.iter().next().unwrap().0.get();
	let max = histogram.iter().next_back().unwrap().0.get();
	let total_values_count = total_values_count.to_f32().unwrap();
	let quantiles: Vec<f32> = vec![0.25, 0.50, 0.75];
	// find the index of each quantile given the total number of values in the dataset
	let quantile_indexes: Vec<usize> = quantiles
		.iter()
		.map(|q| ((total_values_count - 1.0) * q).trunc().to_usize().unwrap())
		.collect();
	// the fractiononal part of the index
	// used to interpolate values if the index is not an integer value
	let quantile_fracts: Vec<f32> = quantiles
		.iter()
		.map(|q| ((total_values_count - 1.0) * q).fract())
		.collect();
	let mut quantiles: Vec<Option<f32>> = vec![None; quantiles.len()];
	let mut current_count: usize = 0;
	let mut mean = 0.0;
	let mut m2 = 0.0;
	let mut iter = histogram.iter().peekable();
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
						// interpolate between two values
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
		progress();
	}
	let quantiles: Vec<f32> = quantiles.into_iter().map(|q| q.unwrap()).collect();
	NumberHistogramStats {
		p25: quantiles[0],
		p50: quantiles[1],
		p75: quantiles[2],
		min,
		max,
		mean: mean.to_f32().unwrap(),
		variance: metrics::m2_to_variance(
			m2,
			NonZeroU64::new(current_count.to_u64().unwrap()).unwrap(),
		),
	}
}

impl ColumnStatsOutput {
	/// Return the name of the source column.
	pub fn column_name(&self) -> &str {
		match self {
			Self::Unknown(value) => &value.column_name,
			Self::Text(value) => &value.column_name,
			Self::Number(value) => &value.column_name,
			Self::Enum(value) => &value.column_name,
		}
	}
	/// Return an option of the number of unique values in this column.
	pub fn unique_values(&self) -> Option<Vec<String>> {
		match self {
			Self::Unknown(_) => None,
			Self::Text(_) => None,
			Self::Number(stats) => stats.histogram.as_ref().map(|histogram| {
				let mut unique_values: Vec<_> = histogram
					.iter()
					.map(|(value, _)| value.to_string())
					.collect();
				unique_values.sort_unstable();
				unique_values
			}),
			Self::Enum(stats) => {
				let mut unique_values: Vec<_> = stats
					.histogram
					.iter()
					.map(|(value, _)| value.clone())
					.collect();
				unique_values.sort_unstable();
				Some(unique_values)
			}
		}
	}
}

#[test]
fn test_compute_number_histogram_stats_one() {
	let mut histogram = BTreeMap::new();
	histogram.insert(Finite::new(1.0).unwrap(), 1);
	let left = compute_number_histogram_stats(&histogram, 1, || {});
	let right = NumberHistogramStats {
		min: 1.0,
		max: 1.0,
		p25: 1.0,
		p50: 1.0,
		p75: 1.0,
		mean: 1.0,
		variance: 0.0,
	};
	assert_eq!(left, right);
}

#[test]
fn test_compute_number_histogram_stats_two() {
	let mut histogram = BTreeMap::new();
	histogram.insert(Finite::new(1.0).unwrap(), 1);
	histogram.insert(Finite::new(2.0).unwrap(), 1);
	let left = compute_number_histogram_stats(&histogram, 2, || {});
	let right = NumberHistogramStats {
		min: 1.0,
		max: 2.0,
		p25: 1.25,
		p50: 1.5,
		p75: 1.75,
		mean: 1.5,
		variance: 0.25,
	};
	assert_eq!(left, right);
}

#[test]
fn test_compute_number_histogram_stats_multiple() {
	let mut histogram = BTreeMap::new();
	histogram.insert(Finite::new(1.0).unwrap(), 3);
	histogram.insert(Finite::new(2.0).unwrap(), 1);
	let left = compute_number_histogram_stats(&histogram, 4, || {});
	let right = NumberHistogramStats {
		min: 1.0,
		max: 2.0,
		p25: 1.0,
		p50: 1.0,
		p75: 1.25,
		mean: 1.25,
		variance: 0.1875,
	};
	assert_eq!(left, right);
}
