/*!
This module implements Tangram's feature engineering that prepares datasets for machine learning.
*/

use crate::{model, stats};
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
