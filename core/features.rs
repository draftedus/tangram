/*!
This module defines feature groups that are used to transform raw data into values used by the [linear](../linear/index.html) and [tree](../tree/index.html) models.
*/

use crate::stats;
use itertools::izip;
use ndarray::{prelude::*, s};
use tangram_dataframe::*;

/**
A FeatureGroup is an enum describing the type of feature.

The following table shows which combinations or dataframe columns, feature groups, and models are used.

| Source Column                                         | Feature Group                 | Model Types    |
|-------------------------------------------------------|-------------------------------|---------------|
| [NumberColumn](../dataframe/struct.NumberColumn.html) | [NormalizedFeatureGroup](struct.NormalizedFeatureGroup.html) | [linear](../linear/index.html) |
| [NumberColumn](../dataframe/struct.NumberColumn.html) | [IdentifyFeatureGroup](struct.IdentityFeatureGroup.html) | [tree](../tree/index.html) |
| [EnumColumn](../dataframe/struct.EnumColumn.html) | [OneHotEncodedFeatureGroup](struct.OneHotEncodedFeatureGroup.html) | [linear](../linear/index.html) |
| [EnumColumn](../dataframe/struct.EnumColumn.html) | [IdentifyFeatureGroup](struct.IdentityFeatureGroup.html) | [tree](../tree/index.html) |
| [TextColumn](../dataframe/struct.TextColumn.html) | [BagOfWordsFeatureGroup](struct.BagOfWordsFeatureGroup.html) | [linear](../linear/index.html), [tree](../tree/index.html) |
*/
#[derive(Debug)]
pub enum FeatureGroup {
	Identity(IdentityFeatureGroup),
	Normalized(NormalizedFeatureGroup),
	OneHotEncoded(OneHotEncodedFeatureGroup),
	BagOfWords(BagOfWordsFeatureGroup),
}

/**
An IdentityFeatureGroup describes an *identity* mapped feature. The IdentityFeatureGroup is only used for `NumberColumn`s and `EnumColumn`s.

# Example
1. **Source Column Type**: [NumberColumn](../dataframe/struct.NumberColumn.html).

| dataframe data  | feature values  |
|-----------------|-----------------|
| 0.2             | 0.2             |
| 3.0             | 3.0             |
| 2.1             | 2.1             |

2. **Source Column Type**: [EnumColumn](../dataframe/struct.EnumColumn.html).

The source column:
```
EnumColumn {
  name: "color",
  options: ["red", "green", "blue"],
  data: [1, 0, 1, 3],
}
```

| value       | encoding |
|-------------|----------|
| \<MISSING\> | 0        |
| red         | 1        |
| green       | 2        |
| blue        | 3        |

| original data in csv                  | dataframe [data](../dataframe/struct.EnumColumn.html#structfield.data) | feature values |
|---------------------------------------|------------------------------------------------------------------------|----------------|
| "red"                                 | 1                                                                      | 1              |
|\<MISSING\>                            | 0                                                                      | 0              |
| "red"                                 | 1                                                                      | 1              |
| "blue"                                | 3                                                                      | 3              |
*/
#[derive(Debug)]
pub struct IdentityFeatureGroup {
	/// The name of the column in the source dataset.
	pub source_column_name: String,
}

/** A NormalizedFeatureGroup describes a *normalized* feature. The NormalizedFeatureGroup is only used for NumberColumns.

Raw values will be normalized by the mean and standard deviation of the column. See [Z-score Normalization](https://en.wikipedia.org/wiki/Feature_scaling#Standardization_(Z-score_Normalization)).

# Example
```
values: [0.0, 5.2, 1.3, 10.0]
```
**mean**: 2.16667

**std**: 2.70617

`feature_value =  (value - mean) / std`

| dataframe value | feature value                         |
|-----------------|---------------------------------------|
| 0.0             | (0.0 - 2.16667) / 2.70617  = -0.80064 |
| 5.2             | (5.2 - 2.16667) / 2.70617  = 1.12089  |
| 1.3             | (1.3 - 2.16667) / 2.70617  = -0.32026 |
| 10.0            | (10.0 - 2.16667) / 2.70617 = 2.89462  |

*/
#[derive(Debug)]
pub struct NormalizedFeatureGroup {
	/// The name of the column in the source dataset.
	pub source_column_name: String,
	pub mean: f32,
	pub variance: f32,
}

/** A OneHotEncodedFeatureGroup describes a *one-hot-encoded* feature.

For each variant in the raw data, a new *feature* will be created whose value is 1 if the raw data's value is equal to this variant and 0 otherwise. It is called *one-hot* because for every source column, only one of the `n` generated features will have a value of 1.

OneHotEncodedFeatureGroups are used for transforming EnumColumns into features for linear models.

# Example
```
EnumColumn {
  name: "color",
  options: ["red", "green", "blue"]
  data: [3, 2, 0, 1]
}
```

We generate a total of 3 features, one for each of the enum options.

| value  | index |
|--------|-------|
| red    | 0     |
| green  | 1     |
| blue   | 2     |

| original data in csv                  | dataframe [data](../dataframe/struct.EnumColumn.html#structfield.data) | features (3)  |
|---------------------------------------|------------------------------------------------------------------------|---------------|
| "blue"                                | 3                                                                      | [0, 0, 1]     |
| "green"                               | 2                                                                      | [0, 1, 0]     |
| \<MISSING\>                           | 0                                                                      | [0, 0, 0]     |
| "red"                                 | 1                                                                      | [1, 0, 0]     |

Unlike in the dataframe case, we don't need a special feature for "missing", because the all 0's vector encodes this.

*/
#[derive(Debug)]
pub struct OneHotEncodedFeatureGroup {
	/// The name of the column in the source dataset.
	pub source_column_name: String,
	pub categories: Vec<String>,
}

/** A BagOfWordsFeatureGroup describes a text feature that is transformed using the *bag-of-words* method. The source column is always a [TextColumn](../dataframe/struct.TextColumn.html).

The raw text value is tokenized. There are `n` features, one for each token found in the training dataset.
A feature will have a value of _count_*_idf_ if it appears in the raw text and 0 otherwise, where *count* is the number of times the token appears in the raw text and *idf* is the inverse document frequency. See [tf-idf](https://en.wikipedia.org/wiki/Tf%E2%80%93idf).

# Example
**Source Column Type**: [TextColumn](../dataframe/struct.TextColumn.html).
```
TextColumn {
  name: "book_titles",
  data: ["The Little Prince", "Stuart Little", "The Cat in the Hat"]
}
```

In computing stats, we computed the [idf](https://en.wikipedia.org/wiki/Tf%E2%80%93idf#Inverse_document_frequency) score for each token and assigned it a unique index:

| token    | index | idf      |
|----------|-------|----------|
| "cat"    | 0     | log(1/3) |
| "hat"    | 1     | log(1/3) |
| "in"     | 2     | log(1/3) |
| "little" | 3     | log(2/3) |
| "prince" | 4     | log(1/3) |
| "stuart" | 5     | log(1/3) |
| "the"    | 6     | log(2/3) |


We generated one feature for every token in the vocabulary where the feature index corresponds to the index of the token in the previous map. E.g. the bag of words feature at index 1 corresponds to the token "cat", and the bag of words feature at index 5 corresponds to the token "prince".

The feature value is computed using the tf-idf formula: `value = tf * idf`.

**Term Frequency (tf)**
The term frequency, `tf` is 1 if the token appears in the data for this example and 0 otherwise.

**Inverse Document Frequency (idf)**
The idf was computed during stats and is a score that downweights frequently occurring terms. It is computed for each term by taking the log of the ratio of the total number of *documents*, which in our case is the number of training examples and the number of *documents* that contain our particular term. For example, the token "little" appears in 2 example rows and there are 3 total examples in our dataset so its idf score is log(3/2).

| dataframe data       | tokens                             | features (6)                 |
|----------------------|------------------------------------|---------------------------------------------------|
| "The Little Prince"  | ["the", "little", "prince"]        | [0, 0, 0, log(3/2), log(3/1), 0, log(3/2)]        |
| "Stuart Little"      | ["stuart", "little"]               | [0, 0, 0, log(3/2), 0, log(3/1), 0]               |
| "The Cat in the Hat" | ["the", "cat", "in", "the", "hat"] | [log(3/1), log(3/1), log(3/1), 0, 0, 0, log(3/2)] |

*/
#[derive(Debug)]
pub struct BagOfWordsFeatureGroup {
	/// The name of the column in the source dataset.
	pub source_column_name: String,
	/// The tokenizer used to split the text into individual tokens.
	pub tokenizer: Tokenizer,
	/// The tokens produced by splitting the text using the tokenizer.
	/// The second value in the tuple is the [inverse document frequency](https://en.wikipedia.org/wiki/Tf%E2%80%93idf).
	pub tokens: Vec<(String, f32)>,
}

/// A Tokenizer describes how raw text is transformed into tokens.
#[derive(Debug)]
pub enum Tokenizer {
	/// See [AlphanumericTokenizer](../util/text/struct.AlphanumericTokenizer.html).
	Alphanumeric,
}

impl FeatureGroup {
	/// The number of features described by this feature group. For example, OneHotEncoded features generate one feature for every category in the source column and BagOfWords features generate one feature for every token in the vocabulary of the source column. Identity and Normalized features generate a single feature.
	pub fn n_features(&self) -> usize {
		match self {
			Self::Identity(_) => 1,
			Self::Normalized(_) => 1,
			Self::OneHotEncoded(f) => f.categories.len() + 1,
			Self::BagOfWords(f) => f.tokens.len(),
		}
	}
}

/// Compute feature groups for [linear](../linear/index.html) models.
///
/// The difference between this function and [compute_feature_groups_tree](fn.compute_feature_groups_tree.html) is that tree models have native support for enum columns and they do not require that number columns are normalized.
pub fn compute_feature_groups_linear(
	column_stats: &[stats::ColumnStatsOutput],
) -> Vec<FeatureGroup> {
	let mut result = Vec::new();
	for column_stats in column_stats.iter() {
		match column_stats {
			stats::ColumnStatsOutput::Unknown(_) => {}
			stats::ColumnStatsOutput::Number(_) => {
				result.push(compute_normalized_feature_group(column_stats));
			}
			stats::ColumnStatsOutput::Enum(_) => {
				result.push(compute_one_hot_encoded_feature_group(column_stats));
			}
			stats::ColumnStatsOutput::Text(_) => {
				result.push(compute_bag_of_words_feature_group(column_stats))
			}
		};
	}
	result
}

/// Compute feature groups for [tree](../tree/index.html) models.
///
/// The difference between this function and [compute_feature_groups_linear](fn.compute_feature_groups_linear.html) is that tree models have native support for enum columns and they do not require that number columns are normalized.
/// The [FeatureGroups](enum.FeatureGroup.html) for enum and number columns are [IdentityFeatureGroups](struct.IdentityFeatureGroup.html).
pub fn compute_feature_groups_tree(column_stats: &[stats::ColumnStatsOutput]) -> Vec<FeatureGroup> {
	let mut result = Vec::new();
	for column_stats in column_stats.iter() {
		match column_stats {
			stats::ColumnStatsOutput::Unknown(_) => {}
			stats::ColumnStatsOutput::Number(_) => {
				result.push(compute_identity_feature_group(
					column_stats.column_name().to_owned(),
				));
			}
			stats::ColumnStatsOutput::Enum(_) => {
				result.push(compute_identity_feature_group(
					column_stats.column_name().to_owned(),
				));
			}
			stats::ColumnStatsOutput::Text(_) => {
				result.push(compute_bag_of_words_feature_group(column_stats))
			}
		};
	}
	result
}

/// Create an IdentifyFeatureGroup.
fn compute_identity_feature_group(source_column_name: String) -> FeatureGroup {
	FeatureGroup::Identity(IdentityFeatureGroup { source_column_name })
}

/// Create a NormalizedFeatureGroup. This function uses the mean and variance from the [ColumnStats](../stats/struct.ColumnStats.html).
fn compute_normalized_feature_group(column_stats: &stats::ColumnStatsOutput) -> FeatureGroup {
	let column_stats = match &column_stats {
		stats::ColumnStatsOutput::Number(column_stats) => column_stats,
		_ => unreachable!(),
	};
	FeatureGroup::Normalized(NormalizedFeatureGroup {
		source_column_name: column_stats.column_name.to_owned(),
		mean: column_stats.mean,
		variance: column_stats.variance,
	})
}

/// Create a OneHotEncodedFeatureGroup. This function uses the categories taken from the [ColumnStats](../stats/struct.ColumnStats.html).
fn compute_one_hot_encoded_feature_group(column_stats: &stats::ColumnStatsOutput) -> FeatureGroup {
	let categories = match column_stats {
		stats::ColumnStatsOutput::Enum(stats) => {
			let mut unique_values: Vec<_> = stats
				.histogram
				.iter()
				.map(|(value, _)| value.clone())
				.collect();
			unique_values.sort_unstable();
			unique_values
		}
		_ => unreachable!(),
	};
	FeatureGroup::OneHotEncoded(OneHotEncodedFeatureGroup {
		source_column_name: column_stats.column_name().to_owned(),
		categories,
	})
}

/// Create a BagOfWordsFeatureGroup.
fn compute_bag_of_words_feature_group(column_stats: &stats::ColumnStatsOutput) -> FeatureGroup {
	let column_stats = match &column_stats {
		stats::ColumnStatsOutput::Text(column_stats) => column_stats,
		_ => unreachable!(),
	};
	let mut tokens = column_stats
		.top_tokens
		.iter()
		.map(|token| {
			(
				token.token.to_owned(),
				tangram_text::compute_idf(token.examples_count, column_stats.count),
			)
		})
		.collect::<Vec<(String, f32)>>();
	tokens.sort_by(|(a, _), (b, _)| a.cmp(b));
	FeatureGroup::BagOfWords(BagOfWordsFeatureGroup {
		source_column_name: column_stats.column_name.to_owned(),
		tokenizer: Tokenizer::Alphanumeric,
		tokens,
	})
}

/// Compute features given the original data `dataframe` and a slice of [FeatureGroup](enum.FeatureGroup.html) one for each column in the dataframe. The resulting features are placed into the passed in `features` array. `progress` is used to keep track of the progress of this function. This function is used to compute features for **training** [linear](../linear/index.html) models.
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

/// Compute normalized features given a NormalizedFeatureGroup and `dataframe` with the original data. The result is placed into the passed in `features`.
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

/// Compute one hot encoded features given a OneHotEncodedFeatureGroup and `dataframe` with the original data. The result is placed into the passed in `features`.
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

/// Compute bag of words encoded features given a BagOfWordsFeatureGroup and `dataframe` with the original data. The result is placed into the passed in `features`.
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
				let tokenizer = tangram_text::AlphanumericTokenizer;
				let tokens = tokenizer.tokenize(value);
				let bigrams = tangram_text::bigrams(&tokens);
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

/// Compute features given the original data `dataframe` and a slice of [FeatureGroup](enum.FeatureGroup.html) one for each column in the dataframe. The function returns a new DataFrame with the computed features. A `progress` function is passed in and called to track progress of the function. This function is used to compute features for training [tree](../tree/index.html) models.
pub fn compute_features_dataframe(
	dataframe: &DataFrameView,
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

/// Compute bag of words encoded features given a BagOfWordsFeatureGroup, `dataframe` with the original data. The returned Vec<Column> has length equal to the number of tokens in the original column for which the feature group is for.
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
				let tokenizer = tangram_text::AlphanumericTokenizer;
				let tokens = tokenizer.tokenize(value);
				let bigrams = tangram_text::bigrams(&tokens);
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

/// Compute features given the original data `dataframe` and a slice of [FeatureGroup](enum.FeatureGroup.html) one for each column in the dataframe. The function returns an Array of [Value](../dataframe/enum.Value.html). The `progress` closure is called to track progress through the function. This function is used to compute features for making **predictions** with [tree](../tree/index.html) models.
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

/// Compute identity features given a IdentityFeatureGroup and `dataframe` with the original data. The result is placed into the passed in `features`.
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
			izip!(features.column_mut(0), c.data).for_each(|(feature_column, column_value)| {
				*feature_column = Value::Number(*column_value);
				progress()
			});
		}
		ColumnView::Enum(c) => {
			izip!(features.column_mut(0), c.data).for_each(|(feature_column, column_value)| {
				*feature_column = Value::Enum(*column_value);
				progress()
			});
		}
		ColumnView::Text(_) => unimplemented!(),
	}
}

/// Compute bag of words encoded features given a BagOfWordsFeatureGroup and `dataframe` with the original data. The result is placed into the passed in `features`.
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
				let tokenizer = tangram_text::AlphanumericTokenizer;
				let tokens = tokenizer.tokenize(value);
				let bigrams = tangram_text::bigrams(&tokens);
				let mut total = 0.0;
				for token in tokens.iter().chain(bigrams.iter()) {
					if let Ok(index) = feature_group
						.tokens
						.binary_search_by(|(t, _)| t.cmp(&token))
					{
						let value = features
							.get_mut([example_index, index])
							.unwrap()
							.as_number_mut()
							.unwrap();
						let idf = feature_group.tokens[index].1;
						let feature_value = 1.0 * idf;
						total += feature_value.powi(2);
						*value += 1.0 * idf;
					}
				}
				if total > 0.0 {
					let mut feature_row = features.slice_mut(s![example_index, ..]);
					for f in feature_row.iter_mut() {
						*f.as_number_mut().unwrap() /= total;
					}
				}
			}
		}
		progress()
	}
}
