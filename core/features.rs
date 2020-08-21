use crate::{dataframe::*, stats, util::text};
use ndarray::{prelude::*, s, Zip};

#[derive(Debug)]
pub enum FeatureGroup {
	Identity(IdentityFeatureGroup),
	Normalized(NormalizedFeatureGroup),
	OneHotEncoded(OneHotEncodedFeatureGroup),
	BagOfWords(BagOfWordsFeatureGroup),
}

#[derive(Debug)]
pub struct IdentityFeatureGroup {
	pub source_column_name: String,
}

#[derive(Debug)]
pub struct NormalizedFeatureGroup {
	pub source_column_name: String,
	pub mean: f32,
	pub variance: f32,
}

#[derive(Debug)]
pub struct OneHotEncodedFeatureGroup {
	pub source_column_name: String,
	pub categories: Vec<String>,
}

#[derive(Debug)]
pub struct BagOfWordsFeatureGroup {
	pub source_column_name: String,
	pub tokenizer: Tokenizer,
	pub tokens: Vec<(String, f32)>,
}

#[derive(Debug)]
pub enum Tokenizer {
	Alphanumeric,
}

impl FeatureGroup {
	pub fn n_features(&self) -> usize {
		match self {
			Self::Identity(_) => 1,
			Self::Normalized(_) => 1,
			Self::OneHotEncoded(f) => f.categories.len() + 1,
			Self::BagOfWords(f) => f.tokens.len(),
		}
	}
}

pub fn compute_feature_groups_linear(column_stats: &[stats::ColumnStats]) -> Vec<FeatureGroup> {
	let mut result = Vec::new();
	for column_stats in column_stats.iter() {
		match column_stats {
			stats::ColumnStats::Unknown(_) => {}
			stats::ColumnStats::Number(_) => {
				result.push(compute_normalized_feature_group(column_stats));
			}
			stats::ColumnStats::Enum(_) => {
				result.push(compute_one_hot_encoded_feature_group(column_stats));
			}
			stats::ColumnStats::Text(_) => {
				result.push(compute_bag_of_words_feature_group(column_stats))
			}
		};
	}
	result
}

pub fn compute_feature_groups_gbt(column_stats: &[stats::ColumnStats]) -> Vec<FeatureGroup> {
	let mut result = Vec::new();
	for column_stats in column_stats.iter() {
		match column_stats {
			stats::ColumnStats::Unknown(_) => {}
			stats::ColumnStats::Number(_) => {
				result.push(compute_identity_feature_group(
					column_stats.column_name().to_owned(),
				));
			}
			stats::ColumnStats::Enum(_) => {
				result.push(compute_identity_feature_group(
					column_stats.column_name().to_owned(),
				));
			}
			stats::ColumnStats::Text(_) => {
				result.push(compute_bag_of_words_feature_group(column_stats))
			}
		};
	}
	result
}

fn compute_identity_feature_group(source_column_name: String) -> FeatureGroup {
	FeatureGroup::Identity(IdentityFeatureGroup { source_column_name })
}

fn compute_normalized_feature_group(column_stats: &stats::ColumnStats) -> FeatureGroup {
	let column_stats = match &column_stats {
		stats::ColumnStats::Number(column_stats) => column_stats,
		_ => unreachable!(),
	};
	FeatureGroup::Normalized(NormalizedFeatureGroup {
		source_column_name: column_stats.column_name.to_owned(),
		mean: column_stats.mean,
		variance: column_stats.variance,
	})
}

fn compute_one_hot_encoded_feature_group(column_stats: &stats::ColumnStats) -> FeatureGroup {
	FeatureGroup::OneHotEncoded(OneHotEncodedFeatureGroup {
		source_column_name: column_stats.column_name().to_owned(),
		categories: column_stats.unique_values().unwrap(),
	})
}

fn compute_bag_of_words_feature_group(column_stats: &stats::ColumnStats) -> FeatureGroup {
	let column_stats = match &column_stats {
		stats::ColumnStats::Text(column_stats) => column_stats,
		_ => unreachable!(),
	};
	let mut tokens: Vec<(String, f32)> = column_stats
		.top_tokens
		.iter()
		.map(|(token, _, idf)| (token.clone(), *idf))
		.collect();
	tokens.sort_by(|(a, _), (b, _)| a.cmp(b));
	FeatureGroup::BagOfWords(BagOfWordsFeatureGroup {
		source_column_name: column_stats.column_name.to_owned(),
		tokenizer: Tokenizer::Alphanumeric,
		tokens,
	})
}

pub fn compute_features_ndarray(
	dataframe: &DataFrameView,
	feature_groups: &[FeatureGroup],
	mut features: ArrayViewMut2<f32>,
	progress: &dyn Fn(),
) {
	let mut feature_index = 0;
	for feature_group in feature_groups.iter() {
		// update for each feature group
		let n_features_in_group = feature_group.n_features();
		let slice = s![.., feature_index..feature_index + n_features_in_group];
		let features = features.slice_mut(slice);
		match &feature_group {
			FeatureGroup::Identity(_) => unimplemented!(),
			FeatureGroup::Normalized(feature_group) => {
				compute_features_normalized_ndarray(dataframe, feature_group, features, progress)
			}
			FeatureGroup::OneHotEncoded(feature_group) => compute_features_one_hot_encoded_ndarray(
				dataframe,
				feature_group,
				features,
				progress,
			),
			FeatureGroup::BagOfWords(feature_group) => {
				compute_features_bag_of_words_ndarray(dataframe, feature_group, features, progress)
			}
		};
		feature_index += n_features_in_group;
	}
}

fn compute_features_normalized_ndarray(
	dataframe: &DataFrameView,
	feature_group: &NormalizedFeatureGroup,
	mut features: ArrayViewMut2<f32>,
	progress: &dyn Fn(),
) {
	let data = dataframe
		.columns
		.iter()
		.find(|column| column.name() == feature_group.source_column_name)
		.unwrap()
		.as_number()
		.unwrap()
		.data;
	for (feature, value) in features.iter_mut().zip(data.iter()) {
		*feature = if value.is_nan() || feature_group.variance == 0.0 {
			0.0
		} else {
			(value - feature_group.mean) / f32::sqrt(feature_group.variance)
		};
		progress()
	}
}

fn compute_features_one_hot_encoded_ndarray(
	dataframe: &DataFrameView,
	feature_group: &OneHotEncodedFeatureGroup,
	mut features: ArrayViewMut2<f32>,
	progress: &dyn Fn(),
) {
	let data = dataframe
		.columns
		.iter()
		.find(|column| column.name() == feature_group.source_column_name)
		.unwrap()
		.as_enum()
		.unwrap()
		.data;
	features.fill(0.0);
	for (mut features, value) in features.genrows_mut().into_iter().zip(data.iter()) {
		features[*value] = 1.0;
		progress();
	}
}

fn compute_features_bag_of_words_ndarray(
	dataframe: &DataFrameView,
	feature_group: &BagOfWordsFeatureGroup,
	mut features: ArrayViewMut2<f32>,
	progress: &dyn Fn(),
) {
	features.fill(0.0);
	let data = dataframe
		.columns
		.iter()
		.find(|column| column.name() == feature_group.source_column_name)
		.unwrap()
		.as_text()
		.unwrap()
		.data;
	for (mut features, value) in features.genrows_mut().into_iter().zip(data.iter()) {
		match feature_group.tokenizer {
			Tokenizer::Alphanumeric => {
				let tokenizer = crate::util::text::AlphanumericTokenizer;
				let tokens = tokenizer.tokenize(value);
				let bigrams = text::bigrams(&tokens);
				let mut total = 0.0;
				for token in tokens.iter().chain(bigrams.iter()) {
					if let Ok(index) = feature_group
						.tokens
						.binary_search_by(|(t, _)| t.cmp(&token))
					{
						let value = 1.0 * feature_group.tokens.get(index).unwrap().1;
						features[index] += value;
						total += value.powi(2);
					}
				}
				if total > 0.0 {
					let norm = total.sqrt();
					features /= norm;
				}
			}
		}
		progress();
	}
}

/// progress gets ticked for each
pub fn compute_features_dataframe<'a>(
	dataframe: &DataFrameView<'a>,
	feature_groups: &[FeatureGroup],
	progress: &dyn Fn(),
) -> DataFrame {
	let mut result = DataFrame { columns: vec![] };
	for feature_group in feature_groups.iter() {
		match &feature_group {
			FeatureGroup::Identity(feature_group) => {
				let column = dataframe
					.columns
					.iter()
					.find(|column| column.name() == feature_group.source_column_name)
					.unwrap();
				let column = match column {
					ColumnView::Unknown(c) => {
						Column::Unknown(UnknownColumn::new(c.name.to_owned()))
					}
					ColumnView::Number(c) => Column::Number(NumberColumn {
						name: c.name.to_owned(),
						data: c.data.to_owned(),
					}),
					ColumnView::Enum(c) => Column::Enum(EnumColumn {
						name: c.name.to_owned(),
						data: c.data.to_owned(),
						options: c.options.to_owned(),
					}),
					ColumnView::Text(c) => Column::Text(TextColumn {
						name: c.name.to_owned(),
						data: c.data.to_owned(),
					}),
				};
				result.columns.push(column);
			}
			FeatureGroup::Normalized(_) => unimplemented!(),
			FeatureGroup::OneHotEncoded(_) => unimplemented!(),
			FeatureGroup::BagOfWords(feature_group) => {
				let columns =
					compute_features_bag_of_words_dataframe(dataframe, feature_group, progress);
				for column in columns {
					result.columns.push(column);
				}
			}
		};
	}
	result
}

fn compute_features_bag_of_words_dataframe(
	dataframe: &DataFrameView,
	feature_group: &BagOfWordsFeatureGroup,
	progress: impl Fn(),
) -> Vec<Column> {
	let data = dataframe
		.columns
		.iter()
		.find(|column| column.name() == feature_group.source_column_name)
		.unwrap()
		.as_text()
		.unwrap()
		.data;

	let mut columns: Vec<NumberColumn> = feature_group
		.tokens
		.iter()
		.map(|token| NumberColumn {
			name: token.0.clone(),
			data: vec![0.0; data.len()],
		})
		.collect();
	for (example_index, value) in data.iter().enumerate() {
		match feature_group.tokenizer {
			Tokenizer::Alphanumeric => {
				let tokenizer = crate::util::text::AlphanumericTokenizer;
				let tokens = tokenizer.tokenize(value);
				let bigrams = text::bigrams(&tokens);
				let mut total = 0.0;
				for token in tokens.iter().chain(bigrams.iter()) {
					if let Ok(index) = feature_group
						.tokens
						.binary_search_by(|(t, _)| t.cmp(&token))
					{
						let idf = feature_group.tokens[index].1;
						let feature_value = 1.0 * idf;
						total += feature_value.powi(2);
						columns[index].data[example_index] += feature_value;
					}
				}
				if total > 0.0 {
					for column in columns.iter_mut() {
						column.data[example_index] /= total;
					}
				}
			}
		}
		progress();
	}
	columns.into_iter().map(Column::Number).collect()
}

pub fn compute_features_ndarray_value(
	dataframe: &DataFrameView,
	feature_groups: &[FeatureGroup],
	mut features: ArrayViewMut2<Value>,
	progress: &dyn Fn(),
) {
	let mut feature_index = 0;
	for feature_group in feature_groups.iter() {
		let n_features_in_group = feature_group.n_features();
		let slice = s![.., feature_index..feature_index + n_features_in_group];
		let features = features.slice_mut(slice);
		match &feature_group {
			FeatureGroup::Identity(feature_group) => compute_features_identity_ndarray_value(
				dataframe,
				feature_group,
				features,
				progress,
			),
			FeatureGroup::Normalized(_) => unimplemented!(),
			FeatureGroup::OneHotEncoded(_) => unimplemented!(),
			FeatureGroup::BagOfWords(feature_group) => compute_features_bag_of_words_ndarray_value(
				dataframe,
				feature_group,
				features,
				progress,
			),
		};
		feature_index += n_features_in_group;
	}
}

fn compute_features_identity_ndarray_value(
	dataframe: &DataFrameView,
	feature_group: &IdentityFeatureGroup,
	mut features: ArrayViewMut2<Value>,
	progress: &dyn Fn(),
) {
	let column = dataframe
		.columns
		.iter()
		.find(|column| column.name() == feature_group.source_column_name)
		.unwrap();
	match column {
		ColumnView::Unknown(_) => unimplemented!(),
		ColumnView::Number(c) => {
			Zip::from(features.column_mut(0))
				.and(c.data)
				.apply(|feature_column, column_value| {
					*feature_column = Value::Number(*column_value);
					progress()
				});
		}
		ColumnView::Enum(c) => {
			Zip::from(features.column_mut(0))
				.and(c.data)
				.apply(|feature_column, column_value| {
					*feature_column = Value::Enum(*column_value);
					progress()
				});
		}
		ColumnView::Text(_) => unimplemented!(),
	}
}

fn compute_features_bag_of_words_ndarray_value(
	dataframe: &DataFrameView,
	feature_group: &BagOfWordsFeatureGroup,
	mut features: ArrayViewMut2<Value>,
	progress: &dyn Fn(),
) {
	let data = dataframe
		.columns
		.iter()
		.find(|column| column.name() == feature_group.source_column_name)
		.unwrap()
		.as_text()
		.unwrap()
		.data;

	features
		.iter_mut()
		.for_each(|feature| *feature = Value::Number(0.0));

	for (example_index, value) in data.iter().enumerate() {
		match feature_group.tokenizer {
			Tokenizer::Alphanumeric => {
				let tokenizer = crate::util::text::AlphanumericTokenizer;
				let tokens = tokenizer.tokenize(value);
				let bigrams = text::bigrams(&tokens);
				let mut total = 0.0;
				for token in tokens.iter().chain(bigrams.iter()) {
					if let Ok(index) = feature_group
						.tokens
						.binary_search_by(|(t, _)| t.cmp(&token))
					{
						// set the feature at index
						let value = features.get_mut([example_index, index]).unwrap();
						let idf = feature_group.tokens[index].1;
						let feature_value = 1.0 * idf;
						total += feature_value.powi(2);
						match value {
							Value::Number(value) => {
								*value += 1.0 * idf;
							}
							_ => unreachable!(),
						};
					}
				}
				if total > 0.0 {
					let mut feature_row = features.slice_mut(s![example_index, ..]);
					feature_row.iter_mut().for_each(|f| match f {
						Value::Number(v) => *v /= total,
						_ => unreachable!(),
					});
				}
			}
		}
		progress()
	}
}
