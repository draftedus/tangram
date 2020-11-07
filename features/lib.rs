use fnv::{FnvBuildHasher, FnvHashMap, FnvHashSet};
use itertools::Itertools;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use tangram_dataframe::{
	DataFrame, DataFrameColumn, DataFrameColumnView, DataFrameValue, DataFrameView,
	EnumDataFrameColumn, NumberDataFrameColumn, TextDataFrameColumn, UnknownDataFrameColumn,
};
use tangram_metrics::Metric;
pub use tangram_util::text::Token;
use tangram_util::{alphanumeric_tokenizer::AlphanumericTokenizer, text::TokenStats, zip};

/// Compute features as an `Array` of `f32`s.
pub fn compute_features_array_f32(
	dataframe: &DataFrameView,
	feature_groups: &[FeatureGroup],
	progress: &impl Fn(),
) -> Array2<f32> {
	let n_features = feature_groups
		.iter()
		.map(|feature_group| feature_group.n_features())
		.sum::<usize>();
	let mut features = Array::zeros((dataframe.nrows(), n_features));
	let mut feature_index = 0;
	for feature_group in feature_groups.iter() {
		let n_features_in_group = feature_group.n_features();
		let slice = s![.., feature_index..feature_index + n_features_in_group];
		let features = features.slice_mut(slice);
		match &feature_group {
			FeatureGroup::Identity(feature_group) => {
				compute_features_identity_array_f32(dataframe, feature_group, features, progress)
			}
			FeatureGroup::Normalized(feature_group) => {
				compute_features_normalized_array_f32(dataframe, feature_group, features, progress)
			}
			FeatureGroup::OneHotEncoded(feature_group) => {
				compute_features_one_hot_encoded_array_f32(
					dataframe,
					feature_group,
					features,
					progress,
				)
			}
			FeatureGroup::BagOfWords(feature_group) => compute_features_bag_of_words_array_f32(
				dataframe,
				feature_group,
				features,
				progress,
			),
		};
		feature_index += n_features_in_group;
	}
	features
}

fn compute_features_identity_array_f32(
	dataframe: &DataFrameView,
	feature_group: &IdentityFeatureGroup,
	features: ArrayViewMut2<f32>,
	progress: &impl Fn(),
) {
	// Get the source column.
	let source_column = dataframe
		.columns()
		.iter()
		.find(|column| column.name() == Some(&feature_group.source_column_name))
		.unwrap();
	feature_group.compute_array_f32(features, source_column.view(), progress);
}

fn compute_features_normalized_array_f32(
	dataframe: &DataFrameView,
	feature_group: &NormalizedFeatureGroup,
	features: ArrayViewMut2<f32>,
	progress: &impl Fn(),
) {
	// Get the source column.
	let source_column = dataframe
		.columns()
		.iter()
		.find(|column| column.name() == Some(&feature_group.source_column_name))
		.unwrap();
	feature_group.compute_array_f32(features, source_column.view(), progress);
}

fn compute_features_one_hot_encoded_array_f32(
	dataframe: &DataFrameView,
	feature_group: &OneHotEncodedFeatureGroup,
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
	feature_group: &BagOfWordsFeatureGroup,
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
	feature_groups: &[FeatureGroup],
	progress: &impl Fn(u64),
) -> DataFrame {
	let mut features = DataFrame::new(Vec::new(), Vec::new());
	for feature_group in feature_groups.iter() {
		match &feature_group {
			FeatureGroup::Identity(feature_group) => {
				let column =
					compute_features_identity_dataframe(dataframe, feature_group, progress);
				features.columns_mut().push(column);
			}
			FeatureGroup::Normalized(_) => unimplemented!(),
			FeatureGroup::OneHotEncoded(_) => unimplemented!(),
			FeatureGroup::BagOfWords(feature_group) => {
				let columns =
					compute_features_bag_of_words_dataframe(dataframe, feature_group, &|| {
						progress(1)
					});
				for column in columns {
					features.columns_mut().push(column);
				}
			}
		};
	}
	features
}

fn compute_features_identity_dataframe(
	dataframe: &DataFrameView,
	feature_group: &IdentityFeatureGroup,
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
	feature_group: &BagOfWordsFeatureGroup,
	progress: &impl Fn(),
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
		progress,
	);
	feature_columns
		.into_iter()
		.map(|feature_column| {
			DataFrameColumn::Number(NumberDataFrameColumn::new(None, feature_column))
		})
		.collect()
}

/// Compute features as a `DataFrame`.
pub fn compute_features_array_value<'a>(
	dataframe: &DataFrameView<'a>,
	feature_groups: &[FeatureGroup],
	progress: &impl Fn(),
) -> Array2<DataFrameValue<'a>> {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = Array::from_elem((dataframe.nrows(), n_features), DataFrameValue::Unknown);
	let mut feature_index = 0;
	for feature_group in feature_groups.iter() {
		let n_features_in_group = feature_group.n_features();
		let slice = s![.., feature_index..feature_index + n_features_in_group];
		let features = features.slice_mut(slice);
		match &feature_group {
			FeatureGroup::Identity(feature_group) => {
				compute_features_identity_array_value(dataframe, feature_group, features, progress)
			}
			FeatureGroup::Normalized(_) => unimplemented!(),
			FeatureGroup::OneHotEncoded(_) => unimplemented!(),
			FeatureGroup::BagOfWords(feature_group) => compute_features_bag_of_words_array_value(
				dataframe,
				feature_group,
				features,
				progress,
			),
		};
		feature_index += n_features_in_group;
	}
	features
}

fn compute_features_identity_array_value(
	dataframe: &DataFrameView,
	feature_group: &IdentityFeatureGroup,
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
	feature_group: &BagOfWordsFeatureGroup,
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
/// This struct describes how to transform one or more columns from the input dataframe to one or more columns in the output features.
#[derive(Debug)]
pub enum FeatureGroup {
	Identity(IdentityFeatureGroup),
	Normalized(NormalizedFeatureGroup),
	OneHotEncoded(OneHotEncodedFeatureGroup),
	BagOfWords(BagOfWordsFeatureGroup),
}

/**
An `IdentityFeatureGroup` describes the simplest possible feature engineering, which passes a single column from the input dataframe to the output features untouched.

# Example
For a number column:

| dataframe value | feature value |
|-----------------|---------------|
| 0.2             | 0.2           |
| 3.0             | 3.0           |
| 2.1             | 2.1           |

For an enum column:

```
use std::num::NonZeroUsize;
use tangram_dataframe::prelude::*;

EnumDataFrameColumn::new(
  Some("color".to_string()),
  vec!["red".to_string(), "green".to_string(), "blue".to_string()],
  vec![None, Some(NonZeroUsize::new(1).unwrap()), Some(NonZeroUsize::new(2).unwrap()), Some(NonZeroUsize::new(3).unwrap())],
);
```

| value       | encoding |
|-------------|----------|
| "INVALID!"  | None     |
| "red"       | Some(1)  |
| "green"     | Some(2)  |
| "blue"      | Some(3)  |

| dataframe value | feature value |
|-----------------|---------------|
| "INVALID!"      | None          |
| "red"           | Some(1)       |
| "green"         | Some(2)       |
| "blue"          | Some(3)       |
*/
#[derive(Debug)]
pub struct IdentityFeatureGroup {
	pub source_column_name: String,
}

impl IdentityFeatureGroup {
	pub fn compute_array_f32(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: DataFrameColumnView,
		progress: &impl Fn(),
	) {
		// Set the feature values to the source column values.
		match column {
			DataFrameColumnView::Unknown(_) => todo!(),
			DataFrameColumnView::Number(column) => {
				for (feature, value) in zip!(features.iter_mut(), column.view().iter()) {
					*feature = *value;
					progress()
				}
			}
			DataFrameColumnView::Enum(column) => {
				for (feature, value) in zip!(features.iter_mut(), column.view().iter()) {
					*feature = value.map(|v| v.get().to_f32().unwrap()).unwrap_or(0.0);
					progress()
				}
			}
			DataFrameColumnView::Text(_) => todo!(),
		}
	}
	pub fn compute_dataframe(
		&self,
		column: DataFrameColumnView,
		progress: &impl Fn(u64),
	) -> DataFrameColumn {
		let column = match column {
			DataFrameColumnView::Unknown(column) => {
				let column = UnknownDataFrameColumn::new(column.name().map(|name| name.to_owned()));
				DataFrameColumn::Unknown(column)
			}
			DataFrameColumnView::Number(column) => {
				DataFrameColumn::Number(NumberDataFrameColumn::new(
					column.name().map(|name| name.to_owned()),
					column.as_slice().to_owned(),
				))
			}
			DataFrameColumnView::Enum(column) => DataFrameColumn::Enum(EnumDataFrameColumn::new(
				column.name().map(|name| name.to_owned()),
				column.options().to_owned(),
				column.as_slice().to_owned(),
			)),
			DataFrameColumnView::Text(column) => DataFrameColumn::Text(TextDataFrameColumn::new(
				column.name().map(|name| name.to_owned()),
				column.as_slice().to_owned(),
			)),
		};
		progress(column.len().to_u64().unwrap());
		column
	}
	pub fn compute_array_value(
		&self,
		mut features: ArrayViewMut2<DataFrameValue>,
		column: DataFrameColumnView,
		progress: &impl Fn(),
	) {
		match column {
			DataFrameColumnView::Unknown(_) => unimplemented!(),
			DataFrameColumnView::Number(column) => {
				for (feature_column, column_value) in zip!(features.column_mut(0), column.iter()) {
					*feature_column = DataFrameValue::Number(*column_value);
					progress()
				}
			}
			DataFrameColumnView::Enum(column) => {
				for (feature_column, column_value) in zip!(features.column_mut(0), column.iter()) {
					*feature_column = DataFrameValue::Enum(*column_value);
					progress()
				}
			}
			DataFrameColumnView::Text(_) => unimplemented!(),
		}
	}
}

/**
A `NormalizedFeatureGroup` transforms a number column to zero mean and unit variance. [Learn more](https://en.wikipedia.org/wiki/Feature_scaling#Standardization_(Z-score_Normalization).

# Example

```
use tangram_dataframe::prelude::*;

NumberDataFrameColumn::new(
  Some("values".to_string()),
  vec![0.0, 5.2, 1.3, 10.0],
);
```

Mean: 2.16667

Standard Deviation: 2.70617

`feature_value =  (value - mean) / std`

| dataframe value | feature value                         |
|-----------------|---------------------------------------|
| 0.0             | (0.0 - 2.16667) / 2.70617  = -0.80064 |
| 5.2             | (5.2 - 2.16667) / 2.70617  = 1.12089  |
| 1.3             | (1.3 - 2.16667) / 2.70617  = -0.32026 |
*/
#[derive(Debug)]
pub struct NormalizedFeatureGroup {
	pub source_column_name: String,
	pub mean: f32,
	pub variance: f32,
}

impl NormalizedFeatureGroup {
	pub fn new(column: DataFrameColumnView) -> NormalizedFeatureGroup {
		match column {
			DataFrameColumnView::Enum(column) => {
				let values = column
					.as_slice()
					.iter()
					.map(|value| {
						value
							.map(|value| value.get().to_f32().unwrap())
							.unwrap_or(0.0)
					})
					.collect::<Vec<_>>();
				let mean_variance = tangram_metrics::MeanVariance::compute(values.as_slice());
				Self {
					source_column_name: column.name().unwrap().to_owned(),
					mean: mean_variance.mean,
					variance: mean_variance.variance,
				}
			}
			DataFrameColumnView::Number(column) => {
				let mean_variance =
					tangram_metrics::MeanVariance::compute(column.view().as_slice());
				Self {
					source_column_name: column.name().unwrap().to_owned(),
					mean: mean_variance.mean,
					variance: mean_variance.variance,
				}
			}
			_ => unimplemented!(),
		}
	}

	pub fn compute_array_f32(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: DataFrameColumnView,
		progress: &impl Fn(),
	) {
		// Set the feature values to the normalized source column values.
		match column {
			DataFrameColumnView::Unknown(_) => todo!(),
			DataFrameColumnView::Number(column) => {
				for (feature, value) in zip!(features.iter_mut(), column.iter()) {
					*feature = if value.is_nan() || self.variance == 0.0 {
						0.0
					} else {
						(*value - self.mean) / f32::sqrt(self.variance)
					};
					progress()
				}
			}
			DataFrameColumnView::Enum(column) => {
				for (feature, value) in zip!(features.iter_mut(), column.iter()) {
					let value = value
						.map(|value| value.get().to_f32().unwrap())
						.unwrap_or(0.0);
					*feature = if value.is_nan() || self.variance == 0.0 {
						0.0
					} else {
						(value - self.mean) / f32::sqrt(self.variance)
					};
					progress()
				}
			}
			DataFrameColumnView::Text(_) => todo!(),
		}
	}
	pub fn compute_dataframe(&self, column: &mut Vec<f32>) {
		// Set the feature values to the normalized source column values.
		for value in column.iter_mut() {
			*value = (*value - self.mean) / f32::sqrt(self.variance)
		}
	}
	pub fn compute_array_value() {
		todo!()
	}
}

/**
A `OneHotEncodedFeatureGroup` creates one number feature for each option in an enum column, plus one number feature for invalid values. For each example, all of the features will have the value 0.0, except the feature corresponding to the column's value, which will have the value 1.0.

# Example

```
use std::num::NonZeroUsize;
use tangram_dataframe::prelude::*;

EnumDataFrameColumn::new(
  Some("color".to_string()),
  vec!["red".to_string(), "green".to_string(), "blue".to_string()],
  vec![None, Some(NonZeroUsize::new(1).unwrap()), Some(NonZeroUsize::new(2).unwrap()), Some(NonZeroUsize::new(3).unwrap())],
);
```

| dataframe value | feature values |
|-----------------|----------------|
| "INVALID!"      | [0, 0, 0]      |
| "red"           | [1, 0, 0]      |
| "green"         | [0, 1, 0]      |
| "blue"          | [0, 0, 1]      |
*/
#[derive(Debug)]
pub struct OneHotEncodedFeatureGroup {
	pub source_column_name: String,
	pub options: Vec<String>,
}

impl OneHotEncodedFeatureGroup {
	pub fn compute_array_f32(
		&self,
		mut features: ArrayViewMut2<f32>,
		values: &[Option<NonZeroUsize>],
		progress: &impl Fn(),
	) {
		// Fill the features with zeros.
		features.fill(0.0);
		// For each example, set the features corresponding to the enum value to one.
		for (mut features, value) in zip!(features.axis_iter_mut(Axis(0)), values.iter()) {
			let feature_index = value.map(|v| v.get()).unwrap_or(0);
			features[feature_index] = 1.0;
			progress();
		}
	}
	pub fn compute_dataframe() {
		todo!()
	}
	pub fn compute_array_value() {
		todo!()
	}
}

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
	pub tokenizer: Tokenizer,
	/// These are the tokens that were produced for the source column in training.
	pub tokens: Vec<BagOfWordsFeatureGroupToken>,
	/// These are the tokens that were produced for the source column in training.
	pub tokens_map: HashMap<Token, usize, FnvBuildHasher>,
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
	pub fn from_tokens(
		source_column_name: String,
		tokenizer: Tokenizer,
		tokens: Vec<BagOfWordsFeatureGroupToken>,
	) -> BagOfWordsFeatureGroup {
		let tokens_map = tokens
			.iter()
			.enumerate()
			.map(|(i, token)| (token.token.clone(), i))
			.collect();
		BagOfWordsFeatureGroup {
			source_column_name,
			tokenizer,
			tokens,
			tokens_map,
		}
	}

	pub fn fit(
		column: DataFrameColumnView,
		settings: FitBagOfWordsFeatureGroupSettings,
	) -> BagOfWordsFeatureGroup {
		let mut token_occurrence_histogram = FnvHashMap::default();
		let mut token_example_histogram = FnvHashMap::default();
		match column {
			DataFrameColumnView::Text(column) => {
				// Collect statistics about the text in the column.
				for value in column.iter() {
					let mut token_set = FnvHashSet::default();
					for unigram in AlphanumericTokenizer::new(value) {
						let unigram = Token::Unigram(unigram.into_owned());
						token_set.insert(unigram.clone());
						*token_occurrence_histogram.entry(unigram).or_insert(0) += 1;
					}
					for (token_a, token_b) in AlphanumericTokenizer::new(value).tuple_windows() {
						let bigram = Token::Bigram(token_a.into_owned(), token_b.into_owned());
						token_set.insert(bigram.clone());
						*token_occurrence_histogram.entry(bigram).or_insert(0) += 1;
					}
					for token in token_set.into_iter() {
						*token_example_histogram.entry(token).or_insert(0) += 1;
					}
				}
				let mut top_tokens = std::collections::BinaryHeap::new();
				for (token, count) in token_occurrence_histogram.iter() {
					top_tokens.push(tangram_util::text::TokenEntry(token.clone(), *count));
				}
				let n_examples = column.len();
				let top_tokens = (0..settings.top_tokens_count)
					.map(|_| top_tokens.pop())
					.filter_map(|token_entry| {
						token_entry.map(|token_entry| (token_entry.0, token_entry.1))
					})
					.map(|(token, count)| {
						let examples_count = token_example_histogram[&token];
						let idf = ((1.0 + n_examples.to_f32().unwrap())
							/ (1.0 + examples_count.to_f32().unwrap()))
						.ln() + 1.0;
						TokenStats {
							token,
							count,
							examples_count,
							idf,
						}
					})
					.collect::<Vec<_>>();
				let tokens = top_tokens
					.iter()
					.map(|token_stats| BagOfWordsFeatureGroupToken {
						token: token_stats.token.clone(),
						idf: token_stats.idf,
					})
					.collect::<Vec<_>>();
				let tokenizer = Tokenizer::Alphanumeric;
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
			_ => unimplemented!(),
		}
	}

	pub fn compute_array_f32(
		&self,
		mut features: ArrayViewMut2<f32>,
		values: &[String],
		progress: &impl Fn(),
	) {
		// Fill the features with zeros.
		features.fill(0.0);
		// Compute the feature values for each example.
		for (example_index, value) in values.iter().enumerate() {
			match self.tokenizer {
				Tokenizer::Alphanumeric => {
					let mut feature_values_sum_of_squares = 0.0;
					// Set the feature value for each token for this example.
					for token in AlphanumericTokenizer::new(value) {
						let token = Token::Unigram(token.into_owned());
						let token_index = self.tokens_map.get(&token);
						if let Some(token_index) = token_index {
							let token = &self.tokens[*token_index];
							let feature_value = 1.0 * token.idf;
							feature_values_sum_of_squares += feature_value * feature_value;
							*features.get_mut([example_index, *token_index]).unwrap() +=
								feature_value;
						}
					}
					for (token_a, token_b) in AlphanumericTokenizer::new(value).tuple_windows() {
						let token = Token::Bigram(token_a.into_owned(), token_b.into_owned());
						let token_index = self.tokens_map.get(&token);
						if let Some(token_index) = token_index {
							let token = &self.tokens[*token_index];
							let feature_value = 1.0 * token.idf;
							feature_values_sum_of_squares += feature_value * feature_value;
							*features.get_mut([example_index, *token_index]).unwrap() +=
								feature_value;
						}
					}
					// Normalize the feature values for this example.
					if feature_values_sum_of_squares > 0.0 {
						let norm = feature_values_sum_of_squares.sqrt();
						for feature in features.row_mut(example_index).iter_mut() {
							*feature /= norm;
						}
					}
				}
			}
			progress();
		}
	}

	pub fn compute_dataframe(
		&self,
		feature_columns: &mut [Vec<f32>],
		values: &[String],
		progress: &impl Fn(),
	) {
		// Compute the feature values for each example.
		for (example_index, value) in values.iter().enumerate() {
			match self.tokenizer {
				Tokenizer::Alphanumeric => {
					let mut feature_values_sum_of_squares = 0.0;
					// Set the feature value for each token for this example.
					for unigram in
						tangram_util::alphanumeric_tokenizer::AlphanumericTokenizer::new(value)
					{
						let token = Token::Unigram(unigram.into_owned());
						let token_index = self.tokens_map.get(&token);
						if let Some(token_index) = token_index {
							let token = &self.tokens[*token_index];
							let feature_value = 1.0 * token.idf;
							feature_values_sum_of_squares += feature_value * feature_value;
							feature_columns[*token_index][example_index] += feature_value;
						}
					}
					for (token_a, token_b) in AlphanumericTokenizer::new(value).tuple_windows() {
						let token = Token::Bigram(token_a.into_owned(), token_b.into_owned());
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
			}
			progress();
		}
	}

	pub fn compute_array_value(
		&self,
		mut features: ArrayViewMut2<DataFrameValue>,
		values: &[String],
		progress: &impl Fn(),
	) {
		// Fill the features with zeros.
		for feature in features.iter_mut() {
			*feature = DataFrameValue::Number(0.0);
		}
		// Compute the feature values for each example.
		for (example_index, value) in values.iter().enumerate() {
			match self.tokenizer {
				Tokenizer::Alphanumeric => {
					let mut feature_values_sum_of_squares = 0.0;
					// Set the feature value for each token for this example.
					for unigram in
						tangram_util::alphanumeric_tokenizer::AlphanumericTokenizer::new(value)
					{
						let token = Token::Unigram(unigram.into_owned());
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
						let token = Token::Bigram(token_a.into_owned(), token_b.into_owned());
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
				}
			}
			progress();
		}
	}
}

#[derive(Debug)]
pub struct BagOfWordsFeatureGroupToken {
	pub token: Token,
	pub idf: f32,
}

/// A Tokenizer describes how raw text is transformed into tokens.
#[derive(Debug)]
pub enum Tokenizer {
	/// This specifies that an [AlphanumericTokenizer](../util/text/struct.AlphanumericTokenizer.html) should be used.
	Alphanumeric,
}

impl FeatureGroup {
	/// Return the number of features this feature group will produce.
	pub fn n_features(&self) -> usize {
		match self {
			FeatureGroup::Identity(_) => 1,
			FeatureGroup::Normalized(_) => 1,
			FeatureGroup::OneHotEncoded(s) => s.options.len() + 1,
			FeatureGroup::BagOfWords(s) => s.tokens.len(),
		}
	}
}
