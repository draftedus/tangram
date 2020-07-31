use self::{dataset::*, histogram::*};
use crate::{
	dataframe::*, progress::StatsProgress, stats, util::progress_counter::ProgressCounter,
};
use num_traits::ToPrimitive;
use rayon::prelude::*;

mod dataset;
mod histogram;

const TOP_TOKENS_COUNT: usize = 20000;
const MIN_DOCUMENT_FREQUENCY: u64 = 2;

#[derive(Clone, Debug, PartialEq)]
pub struct StatsSettings {
	pub text_histogram_max_size: usize,
	pub number_histogram_max_size: usize,
}

pub struct ComputeStatsOutput {
	pub overall_column_stats: Vec<ColumnStats>,
	pub train_column_stats: Vec<ColumnStats>,
	pub test_column_stats: Vec<ColumnStats>,
}

#[derive(Debug)]
pub enum ColumnStats {
	Unknown(UnknownColumnStats),
	Number(NumberColumnStats),
	Enum(EnumColumnStats),
	Text(TextColumnStats),
}

impl ColumnStats {
	pub fn column_name(&self) -> &str {
		match self {
			Self::Unknown(value) => &value.column_name,
			Self::Text(value) => &value.column_name,
			Self::Number(value) => &value.column_name,
			Self::Enum(value) => &value.column_name,
		}
	}
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

#[derive(Debug)]
pub struct UnknownColumnStats {
	pub column_name: String,
	pub count: u64,
}

#[derive(Debug)]
pub struct NumberColumnStats {
	pub column_name: String,
	pub count: u64,
	pub histogram: Option<Vec<(f32, u64)>>,
	pub unique_count: u64,
	pub max: f32,
	pub mean: f32,
	pub min: f32,
	pub invalid_count: u64,
	pub variance: f32,
	pub std: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

#[derive(Debug)]
pub struct EnumColumnStats {
	pub column_name: String,
	pub count: u64,
	pub histogram: Vec<(String, usize)>,
	pub invalid_count: usize,
	pub unique_count: usize,
}

#[derive(Debug)]
pub struct TextColumnStats {
	pub column_name: String,
	pub count: u64,
	pub top_tokens: Vec<(String, u64, f32)>,
}

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
	let progress_counter = ProgressCounter::new(n_cols as u64 * n_rows as u64);
	update_progress(StatsProgress::DatasetStats(progress_counter));
	let train_dataset_stats: Vec<DatasetStats> = dataframe_train
		.columns
		.par_iter()
		.map(|column| DatasetStats::compute(column, &settings))
		.collect();
	let test_dataset_stats: Vec<DatasetStats> = dataframe_test
		.columns
		.par_iter()
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
				count: dataset_stats.count as u64,
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
		count: dataset_stats.count as u64,
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

fn compute_column_stats_enum(
	column_name: &str,
	dataset_stats: &EnumDatasetStats,
	_settings: &StatsSettings,
) -> stats::ColumnStats {
	stats::ColumnStats::Enum(stats::EnumColumnStats {
		column_name: column_name.to_owned(),
		count: dataset_stats.count as u64,
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

fn compute_column_stats_text(
	column_name: &str,
	dataset_stats: &TextDatasetStats,
	_settings: &StatsSettings,
) -> stats::ColumnStats {
	let mut top_tokens = std::collections::BinaryHeap::new();
	for (token, count) in dataset_stats.unigram_histogram.iter() {
		let entry = TokenEntry(token.clone(), *count as u64);
		top_tokens.push(entry);
	}
	for (token, count) in dataset_stats.bigram_histogram.iter() {
		let entry = TokenEntry(token.clone(), *count as u64);
		top_tokens.push(entry);
	}
	let top_tokens = (0..TOP_TOKENS_COUNT)
		.map(|_| top_tokens.pop())
		.filter_map(|token_entry| token_entry.map(|token_entry| (token_entry.0, token_entry.1)))
		.filter_map(|(token, count)| {
			let document_frequency = dataset_stats.per_example_histogram.get(&token).unwrap();
			if *document_frequency >= MIN_DOCUMENT_FREQUENCY as usize {
				// idf = log (n + 1 / (1 + document_frequency))+ 1
				let n_examples = dataset_stats.count;
				let idf =
					((1.0 + n_examples as f32) / (1.0 + (*document_frequency as f32))).ln() + 1.0;
				Some((token, count, idf))
			} else {
				None
			}
		})
		.collect::<Vec<(String, u64, f32)>>();
	stats::ColumnStats::Text(stats::TextColumnStats {
		column_name: column_name.to_owned(),
		count: dataset_stats.count as u64,
		top_tokens,
	})
}
