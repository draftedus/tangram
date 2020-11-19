use fnv::{FnvBuildHasher, FnvHashMap, FnvHashSet};
use itertools::Itertools;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use tangram_dataframe::{
	DataFrameColumn, DataFrameColumnView, DataFrameValue, NumberDataFrameColumn,
	TextDataFrameColumnView,
};
use tangram_util::alphanumeric_tokenizer::AlphanumericTokenizer;

/**
A BagOfWordsFeatureGroup creates features for a text column using the [Bag of Words](https://en.wikipedia.org/wiki/Bag-of-words_model) method.

First, during training all the values for the text column are tokenized. Then, [IDF](https://en.wikipedia.org/wiki/Tf%E2%80%93idf) values are computed for each token. One feature is created for each token. For each example, the feature for each token will have a value equal to the number of occurrences of the token in the text column's value multiplied by the IDF computed for the token during training.

# Example

```
use tangram_dataframe::prelude::*;

TextDataFrameColumn::new(
  Some("book_titles".to_string()),
  vec!["The Little Prince".to_string(), "Stuart Little".to_string(), "The Cat in the Hat".to_string()]
);
```

| token    |  idf      |
|----------|-----------|
| "cat"    |  log(1/3) |
| "hat"    |  log(1/3) |
| "in"     |  log(1/3) |
| "little" |  log(2/3) |
| "prince" |  log(1/3) |
| "stuart" |  log(1/3) |
| "the"    |  log(2/3) |

| dataframe value      | tokens                             | features values                                   |
|----------------------|------------------------------------|---------------------------------------------------|
| "The Little Prince"  | ["the", "little", "prince"]        | [0, 0, 0, log(3/2), log(3/1), 0, log(3/2)]        |
| "Stuart Little"      | ["stuart", "little"]               | [0, 0, 0, log(3/2), 0, log(3/1), 0]               |
| "The Cat in the Hat" | ["the", "cat", "in", "the", "hat"] | [log(3/1), log(3/1), log(3/1), 0, 0, 0, log(3/2)] |
*/
#[derive(Debug)]
pub struct BagOfWordsFeatureGroup {
	/// This is the name of the text column used to compute features with this feature group.
	pub source_column_name: String,
	/// This is the tokenizer used to split the text into tokens.
	pub tokenizer: BagOfWordsFeatureGroupTokenizer,
	/// These are the tokens that were produced for the source column in training.
	pub tokens: Vec<BagOfWordsFeatureGroupTokensEntry>,
	/// These are the tokens that were produced for the source column in training.
	pub tokens_map: HashMap<BagOfWordsFeatureGroupToken, usize, FnvBuildHasher>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BagOfWordsFeatureGroupToken {
	Unigram(String),
	Bigram(String, String),
}

#[derive(Debug)]
pub struct BagOfWordsFeatureGroupTokensEntry {
	pub token: BagOfWordsFeatureGroupToken,
	pub idf: f32,
}

/// A Tokenizer describes how raw text is transformed into tokens.
#[derive(Debug)]
pub enum BagOfWordsFeatureGroupTokenizer {
	/// This specifies that an [`AlphanumericTokenizer`] should be used.
	Alphanumeric,
}

pub struct FitBagOfWordsFeatureGroupSettings {
	pub include_unigrams: bool,
	pub include_bigrams: bool,
	pub top_tokens_count: usize,
}

impl Default for FitBagOfWordsFeatureGroupSettings {
	fn default() -> FitBagOfWordsFeatureGroupSettings {
		FitBagOfWordsFeatureGroupSettings {
			include_unigrams: true,
			include_bigrams: true,
			top_tokens_count: 20000,
		}
	}
}

impl BagOfWordsFeatureGroup {
	pub fn fit(
		column: DataFrameColumnView,
		settings: FitBagOfWordsFeatureGroupSettings,
	) -> BagOfWordsFeatureGroup {
		match column {
			DataFrameColumnView::Text(column) => Self::fit_for_text_column(column, settings),
			_ => unimplemented!(),
		}
	}

	fn fit_for_text_column(
		column: TextDataFrameColumnView,
		settings: FitBagOfWordsFeatureGroupSettings,
	) -> Self {
		#[derive(Clone, Debug, Eq)]
		struct TokenEntry(pub BagOfWordsFeatureGroupToken, pub usize);
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
		let mut token_occurrence_histogram = FnvHashMap::default();
		let mut token_example_histogram = FnvHashMap::default();
		// Collect statistics about the text in the column.
		for value in column.iter() {
			let mut token_set = FnvHashSet::default();
			for unigram in AlphanumericTokenizer::new(value) {
				let unigram = BagOfWordsFeatureGroupToken::Unigram(unigram.into_owned());
				token_set.insert(unigram.clone());
				*token_occurrence_histogram.entry(unigram).or_insert(0) += 1;
			}
			for (token_a, token_b) in AlphanumericTokenizer::new(value).tuple_windows() {
				let bigram =
					BagOfWordsFeatureGroupToken::Bigram(token_a.into_owned(), token_b.into_owned());
				token_set.insert(bigram.clone());
				*token_occurrence_histogram.entry(bigram).or_insert(0) += 1;
			}
			for token in token_set.into_iter() {
				*token_example_histogram.entry(token).or_insert(0) += 1;
			}
		}
		let mut top_tokens = std::collections::BinaryHeap::new();
		for (token, count) in token_occurrence_histogram.iter() {
			top_tokens.push(TokenEntry(token.clone(), *count));
		}
		let n_examples = column.len();
		let tokens = (0..settings.top_tokens_count)
			.map(|_| top_tokens.pop())
			.filter_map(|token_entry| token_entry.map(|token_entry| (token_entry.0, token_entry.1)))
			.map(|(token, _)| {
				let examples_count = token_example_histogram[&token];
				let idf = ((1.0 + n_examples.to_f32().unwrap())
					/ (1.0 + examples_count.to_f32().unwrap()))
				.ln() + 1.0;
				BagOfWordsFeatureGroupTokensEntry { token, idf }
			})
			.collect::<Vec<_>>();
		let tokenizer = BagOfWordsFeatureGroupTokenizer::Alphanumeric;
		let tokens_map = tokens
			.iter()
			.enumerate()
			.map(|(i, token)| (token.token.clone(), i))
			.collect();
		Self {
			source_column_name: column.name().unwrap().to_owned(),
			tokenizer,
			tokens,
			tokens_map,
		}
	}

	pub fn compute_array_f32(
		&self,
		features: ArrayViewMut2<f32>,
		column: DataFrameColumnView,
		progress: &impl Fn(),
	) {
		match column {
			DataFrameColumnView::Unknown(_) => unimplemented!(),
			DataFrameColumnView::Number(_) => unimplemented!(),
			DataFrameColumnView::Enum(_) => unimplemented!(),
			DataFrameColumnView::Text(column) => {
				self.compute_array_f32_for_text_column(features, column, progress)
			}
		}
	}

	fn compute_array_f32_for_text_column(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: TextDataFrameColumnView,
		progress: &impl Fn(),
	) {
		// Fill the features with zeros.
		features.fill(0.0);
		match self.tokenizer {
			BagOfWordsFeatureGroupTokenizer::Alphanumeric => self
				.compute_array_f32_for_text_column_for_alphanumeric_tokenizer(
					features, column, progress,
				),
		}
	}

	fn compute_array_f32_for_text_column_for_alphanumeric_tokenizer(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: TextDataFrameColumnView,
		progress: &impl Fn(),
	) {
		// Compute the feature values for each example.
		for (example_index, value) in column.iter().enumerate() {
			let mut feature_values_sum_of_squares = 0.0;
			// Set the feature value for each token for this example.
			for token in AlphanumericTokenizer::new(value) {
				let token = BagOfWordsFeatureGroupToken::Unigram(token.into_owned());
				let token_index = self.tokens_map.get(&token);
				if let Some(token_index) = token_index {
					let token = &self.tokens[*token_index];
					let feature_value = 1.0 * token.idf;
					feature_values_sum_of_squares += feature_value * feature_value;
					*features.get_mut([example_index, *token_index]).unwrap() += feature_value;
				}
			}
			for (token_a, token_b) in AlphanumericTokenizer::new(value).tuple_windows() {
				let token =
					BagOfWordsFeatureGroupToken::Bigram(token_a.into_owned(), token_b.into_owned());
				let token_index = self.tokens_map.get(&token);
				if let Some(token_index) = token_index {
					let token = &self.tokens[*token_index];
					let feature_value = 1.0 * token.idf;
					feature_values_sum_of_squares += feature_value * feature_value;
					*features.get_mut([example_index, *token_index]).unwrap() += feature_value;
				}
			}
			// Normalize the feature values for this example.
			if feature_values_sum_of_squares > 0.0 {
				let norm = feature_values_sum_of_squares.sqrt();
				for feature in features.row_mut(example_index).iter_mut() {
					*feature /= norm;
				}
			}
			progress();
		}
	}

	pub fn compute_dataframe(
		&self,
		column: DataFrameColumnView,
		progress: &impl Fn(),
	) -> Vec<DataFrameColumn> {
		match column {
			DataFrameColumnView::Unknown(_) => unimplemented!(),
			DataFrameColumnView::Number(_) => unimplemented!(),
			DataFrameColumnView::Enum(_) => unimplemented!(),
			DataFrameColumnView::Text(column) => {
				self.compute_dataframe_for_text_column(column, progress)
			}
		}
	}

	fn compute_dataframe_for_text_column(
		&self,
		column: TextDataFrameColumnView,
		progress: &impl Fn(),
	) -> Vec<DataFrameColumn> {
		match self.tokenizer {
			BagOfWordsFeatureGroupTokenizer::Alphanumeric => {
				self.compute_dataframe_for_text_column_for_alphanumeric_tokenizer(column, progress)
			}
		}
	}

	fn compute_dataframe_for_text_column_for_alphanumeric_tokenizer(
		&self,
		column: TextDataFrameColumnView,
		progress: &impl Fn(),
	) -> Vec<DataFrameColumn> {
		let mut feature_columns = vec![vec![0.0; column.len()]; self.tokens.len()];
		// Compute the feature values for each example.
		for (example_index, value) in column.iter().enumerate() {
			let mut feature_values_sum_of_squares = 0.0;
			// Set the feature value for each token for this example.
			for unigram in tangram_util::alphanumeric_tokenizer::AlphanumericTokenizer::new(value) {
				let token = BagOfWordsFeatureGroupToken::Unigram(unigram.into_owned());
				let token_index = self.tokens_map.get(&token);
				if let Some(token_index) = token_index {
					let token = &self.tokens[*token_index];
					let feature_value = 1.0 * token.idf;
					feature_values_sum_of_squares += feature_value * feature_value;
					feature_columns[*token_index][example_index] += feature_value;
				}
			}
			for (token_a, token_b) in AlphanumericTokenizer::new(value).tuple_windows() {
				let token =
					BagOfWordsFeatureGroupToken::Bigram(token_a.into_owned(), token_b.into_owned());
				let token_index = self.tokens_map.get(&token);
				if let Some(token_index) = token_index {
					let token = &self.tokens[*token_index];
					let feature_value = 1.0 * token.idf;
					feature_values_sum_of_squares += feature_value * feature_value;
					feature_columns[*token_index][example_index] += feature_value;
				}
			}
			// Normalize the feature values for this example.
			if feature_values_sum_of_squares > 0.0 {
				let norm = feature_values_sum_of_squares.sqrt();
				for feature_column in feature_columns.iter_mut() {
					feature_column[example_index] /= norm;
				}
			}
		}
		progress();
		feature_columns
			.into_iter()
			.map(|feature_column| {
				DataFrameColumn::Number(NumberDataFrameColumn::new(None, feature_column))
			})
			.collect()
	}

	pub fn compute_array_value(
		&self,
		features: ArrayViewMut2<DataFrameValue>,
		column: DataFrameColumnView,
		progress: &impl Fn(),
	) {
		match column {
			DataFrameColumnView::Unknown(_) => unimplemented!(),
			DataFrameColumnView::Number(_) => unimplemented!(),
			DataFrameColumnView::Enum(_) => unimplemented!(),
			DataFrameColumnView::Text(column) => {
				self.compute_array_value_for_text_column(features, column, progress)
			}
		}
	}

	fn compute_array_value_for_text_column(
		&self,
		mut features: ArrayViewMut2<DataFrameValue>,
		column: TextDataFrameColumnView,
		progress: &impl Fn(),
	) {
		// Fill the features with zeros.
		for feature in features.iter_mut() {
			*feature = DataFrameValue::Number(0.0);
		}
		match self.tokenizer {
			BagOfWordsFeatureGroupTokenizer::Alphanumeric => {
				self.compute_array_value_for_text_column_for_alphanumeric_tokenizer(
					features, column, progress,
				);
			}
		}
	}

	fn compute_array_value_for_text_column_for_alphanumeric_tokenizer(
		&self,
		mut features: ArrayViewMut2<DataFrameValue>,
		column: TextDataFrameColumnView,
		progress: &impl Fn(),
	) {
		// Compute the feature values for each example.
		for (example_index, value) in column.iter().enumerate() {
			let mut feature_values_sum_of_squares = 0.0;
			// Set the feature value for each token for this example.
			for unigram in tangram_util::alphanumeric_tokenizer::AlphanumericTokenizer::new(value) {
				let token = BagOfWordsFeatureGroupToken::Unigram(unigram.into_owned());
				let token_index = self.tokens_map.get(&token);
				if let Some(token_index) = token_index {
					let token = &self.tokens[*token_index];
					let feature_value = 1.0 * token.idf;
					feature_values_sum_of_squares += feature_value * feature_value;
					*features
						.get_mut([example_index, *token_index])
						.unwrap()
						.as_number_mut()
						.unwrap() += feature_value;
				}
			}
			for (token_a, token_b) in AlphanumericTokenizer::new(value).tuple_windows() {
				let token =
					BagOfWordsFeatureGroupToken::Bigram(token_a.into_owned(), token_b.into_owned());
				let token_index = self.tokens_map.get(&token);
				if let Some(token_index) = token_index {
					let token = &self.tokens[*token_index];
					let feature_value = 1.0 * token.idf;
					feature_values_sum_of_squares += feature_value * feature_value;
					*features
						.get_mut([example_index, *token_index])
						.unwrap()
						.as_number_mut()
						.unwrap() += feature_value;
				}
			}
			// Normalize the feature values for this example.
			if feature_values_sum_of_squares > 0.0 {
				let norm = feature_values_sum_of_squares.sqrt();
				for feature in features.row_mut(example_index).iter_mut() {
					*feature.as_number_mut().unwrap() /= norm;
				}
			}
			progress();
		}
	}
}
