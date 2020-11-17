/*!
This module implements Tangram's feature engineering that prepares datasets for machine learning.
*/

use crate::stats;

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
	let tokens: Vec<tangram_features::BagOfWordsFeatureGroupTokensEntry> = column_stats
		.top_tokens
		.iter()
		.map(
			|token_stats| tangram_features::BagOfWordsFeatureGroupTokensEntry {
				token: match &token_stats.token {
					stats::Token::Unigram(token) => {
						tangram_features::BagOfWordsFeatureGroupToken::Unigram(token.clone())
					}
					stats::Token::Bigram(token_a, token_b) => {
						tangram_features::BagOfWordsFeatureGroupToken::Bigram(
							token_a.clone(),
							token_b.clone(),
						)
					}
				},
				idf: token_stats.idf,
			},
		)
		.collect();
	let tokenizer = match column_stats.tokenizer {
		stats::Tokenizer::Alphanumeric => {
			tangram_features::BagOfWordsFeatureGroupTokenizer::Alphanumeric
		}
	};
	let tokens_map = tokens
		.iter()
		.enumerate()
		.map(|(i, token)| (token.token.clone(), i))
		.collect();
	tangram_features::FeatureGroup::BagOfWords(tangram_features::BagOfWordsFeatureGroup {
		source_column_name: column_stats.column_name.clone(),
		tokenizer,
		tokens,
		tokens_map,
	})
}
