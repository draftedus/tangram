/*!
This module implements Tangram's feature engineering that prepares datasets for machine learning.
*/

use crate::{model, stats};
use ndarray::{prelude::*, s};
use tangram_dataframe::prelude::*;
pub use tangram_features::{
	BagOfWordsFeatureGroup, BagOfWordsFeatureGroupToken, FeatureGroup, IdentityFeatureGroup,
	NormalizedFeatureGroup, OneHotEncodedFeatureGroup, Token, Tokenizer,
};

impl From<stats::Token> for tangram_features::Token {
	fn from(value: stats::Token) -> tangram_features::Token {
		match value {
			stats::Token::Unigram(token) => tangram_features::Token::Unigram(token),
			stats::Token::Bigram(token_a, token_b) => {
				tangram_features::Token::Bigram(token_a, token_b)
			}
		}
	}
}

impl From<model::Token> for tangram_features::Token {
	fn from(value: model::Token) -> tangram_features::Token {
		match value {
			model::Token::Unigram(token) => tangram_features::Token::Unigram(token),
			model::Token::Bigram(token_a, token_b) => {
				tangram_features::Token::Bigram(token_a, token_b)
			}
		}
	}
}

impl Into<model::Token> for tangram_features::Token {
	fn into(self) -> model::Token {
		match self {
			tangram_features::Token::Unigram(token) => model::Token::Unigram(token),
			tangram_features::Token::Bigram(token_a, token_b) => {
				model::Token::Bigram(token_a, token_b)
			}
		}
	}
}

/// Choose feature groups for linear models based on the column stats.
pub fn choose_feature_groups_linear(
	column_stats: &[stats::ColumnStatsOutput],
) -> Vec<tangram_features::FeatureGroup> {
	let mut result = Vec::new();
	for column_stats in column_stats.iter() {
		match column_stats {
			stats::ColumnStatsOutput::Unknown(_) => {}
			stats::ColumnStatsOutput::Number(column_stats) => {
				result.push(normalized_feature_group_for_column(column_stats));
			}
			stats::ColumnStatsOutput::Enum(column_stats) => {
				result.push(one_hot_encoded_feature_group_for_column(column_stats));
			}
			stats::ColumnStatsOutput::Text(column_stats) => {
				result.push(bag_of_words_feature_group_for_column(column_stats))
			}
		};
	}
	result
}

/// Choose feature groups for tree models based on the column stats.
pub fn choose_feature_groups_tree(
	column_stats: &[stats::ColumnStatsOutput],
) -> Vec<tangram_features::FeatureGroup> {
	let mut result = Vec::new();
	for column_stats in column_stats.iter() {
		match column_stats {
			stats::ColumnStatsOutput::Unknown(_) => {}
			stats::ColumnStatsOutput::Number(_) => {
				result.push(identity_feature_group_for_column(column_stats));
			}
			stats::ColumnStatsOutput::Enum(_) => {
				result.push(identity_feature_group_for_column(column_stats));
			}
			stats::ColumnStatsOutput::Text(column_stats) => {
				result.push(bag_of_words_feature_group_for_column(column_stats))
			}
		};
	}
	result
}

fn identity_feature_group_for_column(
	column_stats: &stats::ColumnStatsOutput,
) -> tangram_features::FeatureGroup {
	tangram_features::FeatureGroup::Identity(tangram_features::IdentityFeatureGroup {
		source_column_name: column_stats.column_name().to_owned(),
	})
}

fn normalized_feature_group_for_column(
	column_stats: &stats::NumberColumnStatsOutput,
) -> tangram_features::FeatureGroup {
	tangram_features::FeatureGroup::Normalized(tangram_features::NormalizedFeatureGroup {
		source_column_name: column_stats.column_name.to_owned(),
		mean: column_stats.mean,
		variance: column_stats.variance,
	})
}

fn one_hot_encoded_feature_group_for_column(
	column_stats: &stats::EnumColumnStatsOutput,
) -> tangram_features::FeatureGroup {
	let mut unique_values: Vec<_> = column_stats
		.histogram
		.iter()
		.map(|(value, _)| value.clone())
		.collect();
	unique_values.sort_unstable();
	tangram_features::FeatureGroup::OneHotEncoded(tangram_features::OneHotEncodedFeatureGroup {
		source_column_name: column_stats.column_name.to_owned(),
		options: unique_values,
	})
}

fn bag_of_words_feature_group_for_column(
	column_stats: &stats::TextColumnStatsOutput,
) -> tangram_features::FeatureGroup {
	let mut tokens = column_stats
		.top_tokens
		.iter()
		.map(
			|token_stats| tangram_features::BagOfWordsFeatureGroupToken {
				token: token_stats.token.clone().into(),
				idf: token_stats.idf,
			},
		)
		.collect::<Vec<_>>();
	// Tokens must be sorted because we perform a binary search through them later.
	tokens.sort_by(|a, b| a.token.cmp(&b.token));
	let tokenizer = match column_stats.tokenizer {
		stats::Tokenizer::Alphanumeric => tangram_features::Tokenizer::Alphanumeric,
	};
	let tokens_map = tokens
		.iter()
		.enumerate()
		.map(|(i, token)| (token.token.clone(), i))
		.collect();
	tangram_features::FeatureGroup::BagOfWords(tangram_features::BagOfWordsFeatureGroup {
		source_column_name: column_stats.column_name.to_owned(),
		tokenizer,
		tokens,
		tokens_map,
	})
}

/// Compute features as an `Array` of `f32`s.
pub fn compute_features_array_f32(
	dataframe: &DataFrameView,
	feature_groups: &[tangram_features::FeatureGroup],
	mut features: ArrayViewMut2<f32>,
	progress: &impl Fn(),
) {
	let mut feature_index = 0;
	for feature_group in feature_groups.iter() {
		let n_features_in_group = feature_group.n_features();
		let slice = s![.., feature_index..feature_index + n_features_in_group];
		let features = features.slice_mut(slice);
		match &feature_group {
			tangram_features::FeatureGroup::Identity(feature_group) => {
				compute_features_identity_array_f32(dataframe, feature_group, features, progress)
			}
			tangram_features::FeatureGroup::Normalized(feature_group) => {
				compute_features_normalized_array_f32(dataframe, feature_group, features, progress)
			}
			tangram_features::FeatureGroup::OneHotEncoded(feature_group) => {
				compute_features_one_hot_encoded_array_f32(
					dataframe,
					feature_group,
					features,
					progress,
				)
			}
			tangram_features::FeatureGroup::BagOfWords(feature_group) => {
				compute_features_bag_of_words_array_f32(
					dataframe,
					feature_group,
					features,
					progress,
				)
			}
		};
		feature_index += n_features_in_group;
	}
}

fn compute_features_identity_array_f32(
	dataframe: &DataFrameView,
	feature_group: &tangram_features::IdentityFeatureGroup,
	features: ArrayViewMut2<f32>,
	progress: &impl Fn(),
) {
	// Get the source column.
	let source_column = dataframe
		.columns()
		.iter()
		.find(|column| column.name() == Some(&feature_group.source_column_name))
		.unwrap();
	let source_column = source_column.as_number().unwrap();
	feature_group.compute_array_f32(features, source_column.as_slice(), progress);
}

fn compute_features_normalized_array_f32(
	dataframe: &DataFrameView,
	feature_group: &tangram_features::NormalizedFeatureGroup,
	features: ArrayViewMut2<f32>,
	progress: &impl Fn(),
) {
	// Get the source column.
	let source_column = dataframe
		.columns()
		.iter()
		.find(|column| column.name() == Some(&feature_group.source_column_name))
		.unwrap();
	let source_column = source_column.as_number().unwrap();
	feature_group.compute_array_f32(features, source_column.view().as_slice(), progress);
}

fn compute_features_one_hot_encoded_array_f32(
	dataframe: &DataFrameView,
	feature_group: &tangram_features::OneHotEncodedFeatureGroup,
	features: ArrayViewMut2<f32>,
	progress: &impl Fn(),
) {
	// Get the source column.
	let source_column = dataframe
		.columns()
		.iter()
		.find(|column| column.name() == Some(&feature_group.source_column_name))
		.unwrap();
	let source_column = source_column.as_enum().unwrap();
	feature_group.compute_array_f32(features, source_column.as_slice(), progress);
}

fn compute_features_bag_of_words_array_f32(
	dataframe: &DataFrameView,
	feature_group: &tangram_features::BagOfWordsFeatureGroup,
	features: ArrayViewMut2<f32>,
	progress: &impl Fn(),
) {
	// Get the source column.
	let source_column = dataframe
		.columns()
		.iter()
		.find(|column| column.name() == Some(&feature_group.source_column_name))
		.unwrap();
	let source_column = source_column.as_text().unwrap();
	feature_group.compute_array_f32(features, source_column.view().as_slice(), progress);
}

/// Compute features as a `DataFrame`.
pub fn compute_features_dataframe(
	dataframe: &DataFrameView,
	feature_groups: &[tangram_features::FeatureGroup],
	progress: &impl Fn(u64),
) -> DataFrame {
	let mut result = DataFrame::new(Vec::new(), Vec::new());
	for feature_group in feature_groups.iter() {
		match &feature_group {
			tangram_features::FeatureGroup::Identity(feature_group) => {
				let column =
					compute_features_identity_dataframe(dataframe, feature_group, progress);
				result.columns_mut().push(column);
			}
			tangram_features::FeatureGroup::Normalized(_) => unimplemented!(),
			tangram_features::FeatureGroup::OneHotEncoded(_) => unimplemented!(),
			tangram_features::FeatureGroup::BagOfWords(feature_group) => {
				let columns =
					compute_features_bag_of_words_dataframe(dataframe, feature_group, progress);
				for column in columns {
					result.columns_mut().push(column);
				}
			}
		};
	}
	result
}

fn compute_features_identity_dataframe(
	dataframe: &DataFrameView,
	feature_group: &tangram_features::IdentityFeatureGroup,
	progress: &impl Fn(u64),
) -> DataFrameColumn {
	let column = dataframe
		.columns()
		.iter()
		.find(|column| column.name().unwrap() == feature_group.source_column_name)
		.unwrap();
	feature_group.compute_dataframe(column.view(), progress)
}

fn compute_features_bag_of_words_dataframe(
	dataframe: &DataFrameView,
	feature_group: &tangram_features::BagOfWordsFeatureGroup,
	progress: &impl Fn(u64),
) -> Vec<DataFrameColumn> {
	// Get the data for the source column.
	let source_column = dataframe
		.columns()
		.iter()
		.find(|column| column.name().unwrap() == feature_group.source_column_name)
		.unwrap();
	let source_column = source_column.as_text().unwrap();
	let mut feature_columns = vec![vec![0.0; source_column.len()]; feature_group.tokens.len()];
	feature_group.compute_dataframe(
		feature_columns.as_mut_slice(),
		source_column.view().as_slice(),
		&|| progress(1),
	);
	feature_columns
		.into_iter()
		.map(|feature_column| {
			DataFrameColumn::Number(NumberDataFrameColumn::new(None, feature_column))
		})
		.collect()
}

/// Compute features as an `Array` of `DataFrameValue`s.
pub fn compute_features_array_value(
	dataframe: &DataFrameView,
	feature_groups: &[tangram_features::FeatureGroup],
	mut features: ArrayViewMut2<DataFrameValue>,
	progress: &impl Fn(),
) {
	let mut feature_index = 0;
	for feature_group in feature_groups.iter() {
		let n_features_in_group = feature_group.n_features();
		let slice = s![.., feature_index..feature_index + n_features_in_group];
		let features = features.slice_mut(slice);
		match &feature_group {
			tangram_features::FeatureGroup::Identity(feature_group) => {
				compute_features_identity_array_value(dataframe, feature_group, features, progress)
			}
			tangram_features::FeatureGroup::Normalized(_) => unimplemented!(),
			tangram_features::FeatureGroup::OneHotEncoded(_) => unimplemented!(),
			tangram_features::FeatureGroup::BagOfWords(feature_group) => {
				compute_features_bag_of_words_array_value(
					dataframe,
					feature_group,
					features,
					progress,
				)
			}
		};
		feature_index += n_features_in_group;
	}
}

fn compute_features_identity_array_value(
	dataframe: &DataFrameView,
	feature_group: &tangram_features::IdentityFeatureGroup,
	features: ArrayViewMut2<DataFrameValue>,
	progress: &impl Fn(),
) {
	let column = dataframe
		.columns()
		.iter()
		.find(|column| column.name().unwrap() == feature_group.source_column_name)
		.unwrap();
	feature_group.compute_array_value(features, column.view(), progress);
}

/// Compute the feature values for a `BagOfWordsFeatureGroup` from `dataframe` and write them to `features`.
fn compute_features_bag_of_words_array_value(
	dataframe: &DataFrameView,
	feature_group: &tangram_features::BagOfWordsFeatureGroup,
	features: ArrayViewMut2<DataFrameValue>,
	progress: &impl Fn(),
) {
	// Get the data for the source column.
	let source_column = dataframe
		.columns()
		.iter()
		.find(|column| column.name().unwrap() == feature_group.source_column_name)
		.unwrap();
	let source_column = source_column.as_text().unwrap();
	feature_group.compute_array_value(features, source_column.as_slice(), progress);
}
