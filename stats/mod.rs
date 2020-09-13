use self::{dataset::*, histogram::*};
use crate::{dataframe::*, stats, train::StatsProgress, util::progress_counter::ProgressCounter};
use num_traits::ToPrimitive;

mod dataset;
mod histogram;

const TOP_TOKENS_COUNT: usize = 20000;
const MIN_DOCUMENT_FREQUENCY: u64 = 2;

/// This struct contains settings used to compute stats.
#[derive(Clone, Debug, PartialEq)]
pub struct StatsSettings {
	/// The maximum number of tokens to store in the histogram.
	pub text_histogram_max_size: usize,
	/// The maximum number of unique numeric values to store in the histogram.
	pub number_histogram_max_size: usize,
}

/// This struct is the output from computing stats. It contains stats for the overall dataset and also stats for just the train and test portions.
pub struct ComputeStatsOutput {
	/// The overall column stats contain stats for the whole dataset.
	pub overall_column_stats: Vec<ColumnStats>,
	/// The train column stats contain stats for the train portion of the dataset.
	pub train_column_stats: Vec<ColumnStats>,
	/// The test column stats contain stats for the test portion of the dataset.
	pub test_column_stats: Vec<ColumnStats>,
}

/// An enum describing the different types of column stats.
#[derive(Debug)]
pub enum ColumnStats {
	Unknown(UnknownColumnStats),
	Number(NumberColumnStats),
	Enum(EnumColumnStats),
	Text(TextColumnStats),
}

impl ColumnStats {
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

/// This struct contains stats for unknown columns.
#[derive(Debug)]
pub struct UnknownColumnStats {
	/// The name of the column as it appears in the csv.
	pub column_name: String,
	/// The total number of examples.
	pub count: u64,
}

/// This struct contains stats for number columns.
#[derive(Debug)]
pub struct NumberColumnStats {
	/// The name of the column as it appears in the csv.
	pub column_name: String,
	/// The total number of examples.
	pub count: u64,
	/// A histogram mapping unique values to their counts. It is `None` if the number of unique values exceeds [`number_histogram_max_size`](struct.StatsSettings.html#structfield.number_histogram_max_size).
	pub histogram: Option<Vec<(f32, u64)>>,
	/// The total number of unique values.
	pub unique_count: u64,
	/// The max of the values in the column.
	pub max: f32,
	/// The mean of the values in the column.
	pub mean: f32,
	/// The min of the values in the column.
	pub min: f32,
	/// The total number of invalid values. Invalid values are values that fail to parse as floating point numbers.
	pub invalid_count: u64,
	/// The variance of the values in the column.
	pub variance: f32,
	/// The standard deviation of the values in the column. It is equal to the square root of the variance.
	pub std: f32,
	/// The p25, or 25th-percentile value in the column.
	pub p25: f32,
	/// The p50, or 50th-percentile value in the column. The median.
	pub p50: f32,
	/// The p75, or 75th-percentile value in the column.
	pub p75: f32,
}

/// This struct contains stats for enum columns.
#[derive(Debug)]
pub struct EnumColumnStats {
	/// The name of the column as it appears in the csv.
	pub column_name: String,
	/// The total number of examples.
	pub count: u64,
	/// A histogram mapping unique variants of the enum to the total count of occurrences of the variant in the dataset.
	pub histogram: Vec<(String, usize)>,
	/// The total number of values in the dataset that are invalid. A value is invalid if it is not one of the enum's variants.
	pub invalid_count: usize,
	/// The total number of unique values, excluding invalid values.
	pub unique_count: usize,
}

/// This struct contains stats for text columns.
#[derive(Debug)]
pub struct TextColumnStats {
	/// The name of the column as it appears in the csv.
	pub column_name: String,
	/// The total number of examples.
	pub count: u64,
	/// A vector of the most frequently occurring tokens. It is a tuple where the first entry is the token, the second is the number of times it appears in the dataset and the third is its idf score.
	pub top_tokens: Vec<(String, u64, f32)>,
}

/// Compute stats given a train and test dataframe.
pub fn compute_stats(
	column_names: &[String],
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
	let progress_counter =
		ProgressCounter::new(n_cols.to_u64().unwrap() * n_rows.to_u64().unwrap());
	update_progress(StatsProgress::DatasetStats(progress_counter));
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
	let overall_dataset_stats: Vec<DatasetStats> = train_dataset_stats
		.iter()
		.zip(test_dataset_stats.iter())
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
	let progress_counter = ProgressCounter::new(n);
	update_progress(StatsProgress::HistogramStats(progress_counter.clone()));
	let train_histogram_stats: Vec<HistogramStats> = train_dataset_stats
		.iter()
		.map(|h| compute_histogram_stats(h, || progress_counter.inc(1)))
		.collect();
	let test_histogram_stats: Vec<HistogramStats> = test_dataset_stats
		.iter()
		.map(|h| compute_histogram_stats(h, || progress_counter.inc(1)))
		.collect();
	let overall_histogram_stats: Vec<HistogramStats> = overall_dataset_stats
		.iter()
		.map(|h| compute_histogram_stats(h, || progress_counter.inc(1)))
		.collect();

	// transform histograms and histogram_stats into column_stats
	let train_column_stats = compute_column_stats(
		column_names,
		&train_dataset_stats,
		train_histogram_stats,
		&settings,
	);
	let test_column_stats = compute_column_stats(
		column_names,
		&test_dataset_stats,
		test_histogram_stats,
		&settings,
	);
	let overall_column_stats = compute_column_stats(
		column_names,
		&overall_dataset_stats,
		overall_histogram_stats,
		&settings,
	);

	ComputeStatsOutput {
		overall_column_stats,
		test_column_stats,
		train_column_stats,
	}
}

fn compute_column_stats(
	column_names: &[String],
	dataset_stats: &[DatasetStats],
	histogram_stats: Vec<HistogramStats>,
	settings: &StatsSettings,
) -> Vec<stats::ColumnStats> {
	column_names
		.iter()
		.zip(dataset_stats.iter().zip(histogram_stats.into_iter()))
		.map(|(column_name, (dataset_stats, histogram_stats))| {
			compute_column_stats_for_column(column_name, dataset_stats, histogram_stats, settings)
		})
		.collect()
}

fn compute_column_stats_for_column(
	column_name: &str,
	dataset_stats: &DatasetStats,
	histogram_stats: HistogramStats,
	settings: &StatsSettings,
) -> stats::ColumnStats {
	match (dataset_stats, &histogram_stats) {
		(DatasetStats::Unknown(dataset_stats), _) => {
			stats::ColumnStats::Unknown(stats::UnknownColumnStats {
				column_name: column_name.to_owned(),
				count: dataset_stats.count.to_u64().unwrap(),
			})
		}
		(DatasetStats::Text(dataset_stats), _) => {
			compute_column_stats_text(column_name, dataset_stats, settings)
		}
		(DatasetStats::Number(dataset_stats), HistogramStats::Number(histogram_stats)) => {
			compute_column_stats_number(column_name, dataset_stats, histogram_stats, settings)
		}
		(DatasetStats::Enum(dataset_stats), _) => {
			compute_column_stats_enum(column_name, dataset_stats, settings)
		}
		_ => unreachable!(),
	}
}

/// Compute [ColumnStats](struct.ColumnStats.html) for a number column by combining stats computed in dataset_stats and histogram_stats.
fn compute_column_stats_number(
	column_name: &str,
	dataset_stats: &NumberDatasetStats,
	histogram_stats: &NumberHistogramStats,
	settings: &StatsSettings,
) -> stats::ColumnStats {
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
	stats::ColumnStats::Number(stats::NumberColumnStats {
		column_name: column_name.to_owned(),
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
fn compute_column_stats_enum(
	column_name: &str,
	dataset_stats: &EnumDatasetStats,
	_settings: &StatsSettings,
) -> stats::ColumnStats {
	stats::ColumnStats::Enum(stats::EnumColumnStats {
		column_name: column_name.to_owned(),
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

/// Compute [ColumnStats](struct.ColumnStats.html) for a text column.
fn compute_column_stats_text(
	column_name: &str,
	dataset_stats: &TextDatasetStats,
	_settings: &StatsSettings,
) -> stats::ColumnStats {
	let mut top_tokens = std::collections::BinaryHeap::new();
	for (token, count) in dataset_stats.unigram_histogram.iter() {
		let entry = TokenEntry(token.clone(), count.to_u64().unwrap());
		top_tokens.push(entry);
	}
	for (token, count) in dataset_stats.bigram_histogram.iter() {
		let entry = TokenEntry(token.clone(), count.to_u64().unwrap());
		top_tokens.push(entry);
	}
	let top_tokens = (0..TOP_TOKENS_COUNT)
		.map(|_| top_tokens.pop())
		.filter_map(|token_entry| token_entry.map(|token_entry| (token_entry.0, token_entry.1)))
		.filter_map(|(token, count)| {
			let document_frequency = dataset_stats.per_example_histogram.get(&token).unwrap();
			if *document_frequency >= MIN_DOCUMENT_FREQUENCY.to_usize().unwrap() {
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
	stats::ColumnStats::Text(stats::TextColumnStats {
		column_name: column_name.to_owned(),
		count: dataset_stats.count.to_u64().unwrap(),
		top_tokens,
	})
}
