use crate::{features, model};
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::{
	collections::BTreeMap,
	convert::{TryFrom, TryInto},
};
use tangram_dataframe::prelude::*;
use tangram_util::{error::Result, zip};

#[derive(serde::Deserialize, Debug)]
pub struct PredictOptions {
	pub threshold: f32,
}

impl Default for PredictOptions {
	fn default() -> PredictOptions {
		PredictOptions { threshold: 0.5 }
	}
}

#[derive(serde::Deserialize, Debug)]
pub struct PredictInput(pub Vec<serde_json::Map<String, serde_json::Value>>);

#[derive(serde::Serialize, Debug)]
#[serde(untagged)]
pub enum PredictOutput {
	Regression(Vec<RegressionPredictOutput>),
	BinaryClassification(Vec<BinaryClassificationPredictOutput>),
	MulticlassClassification(Vec<MulticlassClassificationPredictOutput>),
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegressionPredictOutput {
	pub value: f32,
	pub feature_contributions: Option<FeatureContributions>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BinaryClassificationPredictOutput {
	pub class_name: String,
	pub probability: f32,
	pub feature_contributions: Option<FeatureContributions>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MulticlassClassificationPredictOutput {
	pub class_name: String,
	pub probability: f32,
	pub probabilities: BTreeMap<String, f32>,
	pub feature_contributions: Option<BTreeMap<String, FeatureContributions>>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeatureContributions {
	/// The baseline value is the value output by the model for this class before taking into account the feature values.
	pub baseline_value: f32,
	/// The output value is the sum of the baseline value and the feature contribution values of all features.
	pub output_value: f32,
	/// These are the feature contribution values for each feature.
	pub feature_contributions: Vec<FeatureContribution>,
}

#[derive(serde::Serialize, Debug)]
#[serde(tag = "feature_type")]
pub enum FeatureContribution {
	#[serde(rename = "identity")]
	Identity {
		column_name: String,
		feature_contribution_value: f32,
	},
	#[serde(rename = "normalized")]
	Normalized {
		column_name: String,
		feature_contribution_value: f32,
	},
	#[serde(rename = "one_hot_encoded")]
	OneHotEncoded {
		column_name: String,
		option: Option<String>,
		feature_value: bool,
		feature_contribution_value: f32,
	},
	#[serde(rename = "bag_of_words")]
	BagOfWords {
		column_name: String,
		token: Token,
		feature_value: bool,
		feature_contribution_value: f32,
	},
}

#[derive(serde::Serialize, Debug)]
#[serde(untagged)]
pub enum Token {
	Unigram(String),
	Bigram(String, String),
}

impl From<tangram_features::Token> for Token {
	fn from(value: tangram_features::Token) -> Token {
		match value {
			tangram_features::Token::Unigram(token) => Token::Unigram(token),
			tangram_features::Token::Bigram(token_a, token_b) => Token::Bigram(token_a, token_b),
		}
	}
}

impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Token::Unigram(token) => write!(f, "{}", token),
			Token::Bigram(token_a, token_b) => write!(f, "{} {}", token_a, token_b),
		}
	}
}

#[derive(Debug)]
pub enum Model {
	Regressor(Regressor),
	BinaryClassifier(BinaryClassifier),
	MulticlassClassifier(MulticlassClassifier),
}

#[derive(Debug)]
pub struct Regressor {
	pub id: String,
	pub columns: Vec<Column>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: RegressionModel,
}

#[derive(Debug)]
pub struct BinaryClassifier {
	pub id: String,
	pub columns: Vec<Column>,
	pub negative_class: String,
	pub positive_class: String,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: BinaryClassificationModel,
}

#[derive(Debug)]
pub struct MulticlassClassifier {
	pub id: String,
	pub columns: Vec<Column>,
	pub classes: Vec<String>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: MulticlassClassificationModel,
}

#[derive(Debug)]
pub enum RegressionModel {
	Linear(tangram_linear::Regressor),
	Tree(tangram_tree::Regressor),
}

#[derive(Debug)]
pub enum BinaryClassificationModel {
	Linear(tangram_linear::BinaryClassifier),
	Tree(tangram_tree::BinaryClassifier),
}

#[derive(Debug)]
pub enum MulticlassClassificationModel {
	Linear(tangram_linear::MulticlassClassifier),
	Tree(tangram_tree::MulticlassClassifier),
}

#[derive(Debug)]
pub enum Column {
	Unknown(UnknownColumn),
	Number(NumberColumn),
	Enum(EnumColumn),
	Text(TextColumn),
}

#[derive(Debug)]
pub struct UnknownColumn {
	name: String,
}

#[derive(Debug)]
pub struct NumberColumn {
	name: String,
}

#[derive(Debug)]
pub struct EnumColumn {
	name: String,
	options: Vec<String>,
}

#[derive(Debug)]
pub struct TextColumn {
	name: String,
}

pub fn predict(
	model: &Model,
	input: PredictInput,
	options: Option<PredictOptions>,
) -> PredictOutput {
	// Initialize the dataframe.
	let columns = match model {
		Model::Regressor(model) => model.columns.as_slice(),
		Model::BinaryClassifier(model) => model.columns.as_slice(),
		Model::MulticlassClassifier(model) => model.columns.as_slice(),
	};
	let column_names = columns
		.iter()
		.map(|column| match column {
			Column::Unknown(column) => Some(column.name.clone()),
			Column::Number(column) => Some(column.name.clone()),
			Column::Enum(column) => Some(column.name.clone()),
			Column::Text(column) => Some(column.name.clone()),
		})
		.collect();
	let column_types = columns
		.iter()
		.map(|column| match column {
			Column::Unknown(_) => tangram_dataframe::DataFrameColumnType::Unknown,
			Column::Number(_) => tangram_dataframe::DataFrameColumnType::Number,
			Column::Enum(column) => tangram_dataframe::DataFrameColumnType::Enum {
				options: column.options.clone(),
			},
			Column::Text(_) => tangram_dataframe::DataFrameColumnType::Text,
		})
		.collect();
	let mut dataframe = tangram_dataframe::DataFrame::new(column_names, column_types);
	// Fill the dataframe with the input.
	for input in input.0 {
		for column in dataframe.columns_mut().iter_mut() {
			match column {
				tangram_dataframe::DataFrameColumn::Unknown(column) => *column.len_mut() += 1,
				tangram_dataframe::DataFrameColumn::Number(column) => {
					let value = match input.get(column.name().as_ref().unwrap()) {
						Some(serde_json::Value::Number(value)) => {
							value.as_f64().unwrap().to_f32().unwrap()
						}
						_ => std::f32::NAN,
					};
					column.data_mut().push(value);
				}
				tangram_dataframe::DataFrameColumn::Enum(column) => {
					let value = input
						.get(column.name().as_ref().unwrap())
						.and_then(|value| value.as_str());
					let value = value.and_then(|value| column.value_for_option(value));
					column.data_mut().push(value);
				}
				tangram_dataframe::DataFrameColumn::Text(column) => {
					let value = input
						.get(column.name().as_ref().unwrap())
						.and_then(|value| value.as_str())
						.unwrap_or("")
						.to_owned();
					column.data_mut().push(value);
				}
			}
		}
	}
	// Make the predictions by matching on the model type.
	match model {
		Model::Regressor(model) => {
			PredictOutput::Regression(predict_regressor(model, dataframe, options))
		}
		Model::BinaryClassifier(model) => PredictOutput::BinaryClassification(
			predict_binary_classifier(model, dataframe, options),
		),
		Model::MulticlassClassifier(model) => PredictOutput::MulticlassClassification(
			predict_multiclass_classifier(model, dataframe, options),
		),
	}
}

fn predict_regressor(
	model: &Regressor,
	dataframe: DataFrame,
	_options: Option<PredictOptions>,
) -> Vec<RegressionPredictOutput> {
	let n_examples = dataframe.nrows();
	match &model.model {
		RegressionModel::Linear(inner_model) => {
			let mut predictions = Array::zeros(n_examples);
			let features = tangram_features::compute_features_array_f32(
				&dataframe.view(),
				&model.feature_groups,
				&|| {},
			);
			inner_model.predict(features.view(), predictions.view_mut());
			let feature_contributions = inner_model.compute_feature_contributions(features.view());
			zip!(
				features.axis_iter(Axis(0)),
				predictions.iter(),
				feature_contributions,
			)
			.map(|(features, prediction, feature_contributions)| {
				let baseline_value = feature_contributions.baseline_value;
				let output_value = feature_contributions.output_value;
				let feature_contributions = compute_feature_contributions(
					model.feature_groups.iter(),
					features.iter().cloned(),
					feature_contributions
						.feature_contribution_values
						.into_iter(),
				);
				let feature_contributions = FeatureContributions {
					baseline_value,
					output_value,
					feature_contributions,
				};
				RegressionPredictOutput {
					value: *prediction,
					feature_contributions: Some(feature_contributions),
				}
			})
			.collect()
		}
		RegressionModel::Tree(inner_model) => {
			let features = tangram_features::compute_features_array_value(
				&dataframe.view(),
				&model.feature_groups,
				&|| {},
			);
			let mut predictions = Array::zeros(n_examples);
			inner_model.predict(features.view(), predictions.view_mut());
			let feature_contributions = inner_model.compute_feature_contributions(features.view());
			zip!(
				features.axis_iter(Axis(0)),
				predictions.iter(),
				feature_contributions
			)
			.map(|(features, prediction, feature_contributions)| {
				let baseline_value = feature_contributions.baseline_value;
				let output_value = feature_contributions.output_value;
				let feature_contributions = compute_feature_contributions(
					model.feature_groups.iter(),
					features.iter().map(|v| match v {
						tangram_dataframe::DataFrameValue::Number(value) => *value,
						tangram_dataframe::DataFrameValue::Enum(value) => {
							value.map(|v| v.get()).unwrap_or(0).to_f32().unwrap()
						}
						_ => unreachable!(),
					}),
					feature_contributions
						.feature_contribution_values
						.into_iter(),
				);
				let feature_contributions = FeatureContributions {
					baseline_value,
					output_value,
					feature_contributions,
				};
				RegressionPredictOutput {
					value: *prediction,
					feature_contributions: Some(feature_contributions),
				}
			})
			.collect()
		}
	}
}

fn predict_binary_classifier(
	model: &BinaryClassifier,
	dataframe: DataFrame,
	options: Option<PredictOptions>,
) -> Vec<BinaryClassificationPredictOutput> {
	let n_examples = dataframe.nrows();
	match &model.model {
		BinaryClassificationModel::Linear(inner_model) => {
			let mut probabilities = Array::zeros(n_examples);
			let features = tangram_features::compute_features_array_f32(
				&dataframe.view(),
				&model.feature_groups,
				&|| {},
			);
			inner_model.predict(features.view(), probabilities.view_mut());
			let feature_contributions = inner_model.compute_feature_contributions(features.view());
			let threshold = match options {
				Some(options) => options.threshold,
				None => 0.5,
			};
			zip!(probabilities.iter(), feature_contributions)
				.map(|(probability, feature_contributions)| {
					let (probability, class_name) = if *probability >= threshold {
						(*probability, model.positive_class.clone())
					} else {
						(1.0 - probability, model.negative_class.clone())
					};
					let baseline_value = feature_contributions.baseline_value;
					let output_value = feature_contributions.output_value;
					let feature_contributions = compute_feature_contributions(
						model.feature_groups.iter(),
						features.iter().cloned(),
						feature_contributions
							.feature_contribution_values
							.into_iter(),
					);
					let feature_contributions = FeatureContributions {
						baseline_value,
						output_value,
						feature_contributions,
					};
					BinaryClassificationPredictOutput {
						class_name,
						probability,
						feature_contributions: Some(feature_contributions),
					}
				})
				.collect()
		}
		BinaryClassificationModel::Tree(inner_model) => {
			let features = tangram_features::compute_features_array_value(
				&dataframe.view(),
				&model.feature_groups,
				&|| {},
			);
			let mut probabilities = Array::zeros(n_examples);
			inner_model.predict(features.view(), probabilities.view_mut());
			let feature_contributions = inner_model.compute_feature_contributions(features.view());
			let threshold = match options {
				Some(options) => options.threshold,
				None => 0.5,
			};
			zip!(probabilities.iter(), feature_contributions)
				.map(|(probability, feature_contributions)| {
					let (probability, class_name) = if *probability >= threshold {
						(*probability, model.positive_class.clone())
					} else {
						(1.0 - probability, model.negative_class.clone())
					};
					let baseline_value = feature_contributions.baseline_value;
					let output_value = feature_contributions.output_value;
					let feature_contributions = compute_feature_contributions(
						model.feature_groups.iter(),
						features.iter().map(|v| match v {
							tangram_dataframe::DataFrameValue::Number(value) => *value,
							tangram_dataframe::DataFrameValue::Enum(value) => {
								value.map(|v| v.get()).unwrap_or(0).to_f32().unwrap()
							}
							_ => unreachable!(),
						}),
						feature_contributions
							.feature_contribution_values
							.into_iter(),
					);
					let feature_contributions = FeatureContributions {
						baseline_value,
						output_value,
						feature_contributions,
					};
					BinaryClassificationPredictOutput {
						class_name,
						probability,
						feature_contributions: Some(feature_contributions),
					}
				})
				.collect()
		}
	}
}

fn predict_multiclass_classifier(
	model: &MulticlassClassifier,
	dataframe: DataFrame,
	_options: Option<PredictOptions>,
) -> Vec<MulticlassClassificationPredictOutput> {
	let n_examples = dataframe.nrows();
	let n_classes = model.classes.len();
	match &model.model {
		MulticlassClassificationModel::Linear(inner_model) => {
			let mut probabilities = Array::zeros((n_examples, n_classes));
			let features = tangram_features::compute_features_array_f32(
				&dataframe.view(),
				&model.feature_groups,
				&|| {},
			);
			inner_model.predict(features.view(), probabilities.view_mut());
			let feature_contributions = inner_model.compute_feature_contributions(features.view());
			zip!(probabilities.axis_iter(Axis(0)), feature_contributions)
				.map(|(probabilities, feature_contributions)| {
					let (probability, class_name) =
						zip!(probabilities.iter(), model.classes.iter())
							.max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
							.unwrap();
					let probabilities = zip!(probabilities, model.classes.iter())
						.map(|(p, c)| (c.clone(), *p))
						.collect();
					let feature_contributions = zip!(model.classes.iter(), feature_contributions)
						.map(|(class, feature_contributions)| {
							let baseline_value = feature_contributions.baseline_value;
							let output_value = feature_contributions.output_value;
							let feature_contributions = compute_feature_contributions(
								model.feature_groups.iter(),
								features.iter().cloned(),
								feature_contributions
									.feature_contribution_values
									.into_iter(),
							);
							let feature_contributions = FeatureContributions {
								baseline_value,
								output_value,
								feature_contributions,
							};
							(class.clone(), feature_contributions)
						})
						.collect();
					MulticlassClassificationPredictOutput {
						class_name: class_name.clone(),
						probability: *probability,
						probabilities,
						feature_contributions: Some(feature_contributions),
					}
				})
				.collect()
		}
		MulticlassClassificationModel::Tree(inner_model) => {
			let features = tangram_features::compute_features_array_value(
				&dataframe.view(),
				&model.feature_groups,
				&|| {},
			);
			let mut probabilities = Array::zeros((n_examples, n_classes));
			inner_model.predict(features.view(), probabilities.view_mut());
			let feature_contributions = inner_model.compute_feature_contributions(features.view());
			zip!(probabilities.axis_iter(Axis(0)), feature_contributions)
				.map(|(probabilities, feature_contributions)| {
					let (probability, class_name) =
						zip!(probabilities.iter(), model.classes.iter())
							.max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
							.unwrap();
					let probabilities = zip!(probabilities.iter(), model.classes.iter())
						.map(|(probability, class)| (class.clone(), *probability))
						.collect();
					let feature_contributions = zip!(model.classes.iter(), feature_contributions)
						.map(|(class, feature_contributions)| {
							let baseline_value = feature_contributions.baseline_value;
							let output_value = feature_contributions.output_value;
							let feature_contributions = compute_feature_contributions(
								model.feature_groups.iter(),
								features.iter().map(|v| match v {
									tangram_dataframe::DataFrameValue::Number(value) => *value,
									tangram_dataframe::DataFrameValue::Enum(value) => {
										value.map(|v| v.get()).unwrap_or(0).to_f32().unwrap()
									}
									_ => unreachable!(),
								}),
								feature_contributions
									.feature_contribution_values
									.into_iter(),
							);
							let feature_contributions = FeatureContributions {
								baseline_value,
								output_value,
								feature_contributions,
							};
							(class.clone(), feature_contributions)
						})
						.collect();
					MulticlassClassificationPredictOutput {
						class_name: class_name.clone(),
						probability: *probability,
						probabilities,
						feature_contributions: Some(feature_contributions),
					}
				})
				.collect()
		}
	}
}

fn compute_feature_contributions<'a>(
	feature_groups: impl Iterator<Item = &'a features::FeatureGroup>,
	mut features: impl Iterator<Item = f32>,
	mut feature_contribution_values: impl Iterator<Item = f32>,
) -> Vec<FeatureContribution> {
	let mut feature_contributions = Vec::new();
	for feature_group in feature_groups {
		match feature_group {
			features::FeatureGroup::Identity(feature_group) => {
				let _feature_value = features.next().unwrap();
				let feature_contribution_value = feature_contribution_values.next().unwrap();
				feature_contributions.push(FeatureContribution::Identity {
					column_name: feature_group.source_column_name.clone(),
					feature_contribution_value,
				});
			}
			features::FeatureGroup::Normalized(feature_group) => {
				let _feature_value = features.next().unwrap();
				let feature_contribution_value = feature_contribution_values.next().unwrap();
				feature_contributions.push(FeatureContribution::Normalized {
					column_name: feature_group.source_column_name.clone(),
					feature_contribution_value,
				});
			}
			features::FeatureGroup::OneHotEncoded(feature_group) => {
				let feature_value = features.next().unwrap();
				let feature_contribution_value = feature_contribution_values.next().unwrap();
				feature_contributions.push(FeatureContribution::OneHotEncoded {
					column_name: feature_group.source_column_name.clone(),
					option: None,
					feature_value: feature_value > 0.0,
					feature_contribution_value,
				});
				for option in feature_group.options.iter() {
					let feature_value = features.next().unwrap();
					let feature_contribution_value = feature_contribution_values.next().unwrap();
					feature_contributions.push(FeatureContribution::OneHotEncoded {
						column_name: feature_group.source_column_name.clone(),
						option: Some(option.clone()),
						feature_value: feature_value > 0.0,
						feature_contribution_value,
					});
				}
			}
			features::FeatureGroup::BagOfWords(feature_group) => {
				for token in feature_group.tokens.iter() {
					let feature_value = features.next().unwrap();
					let feature_contribution_value = feature_contribution_values.next().unwrap();
					feature_contributions.push(FeatureContribution::BagOfWords {
						column_name: feature_group.source_column_name.clone(),
						token: token.token.clone().into(),
						feature_value: feature_value > 0.0,
						feature_contribution_value,
					});
				}
			}
		}
	}
	feature_contributions
}

impl TryFrom<model::Model> for Model {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::Model) -> Result<Model> {
		match value {
			model::Model::Regressor(model) => Ok(Model::Regressor(model.try_into()?)),
			model::Model::BinaryClassifier(model) => Ok(Model::BinaryClassifier(model.try_into()?)),
			model::Model::MulticlassClassifier(model) => {
				Ok(Model::MulticlassClassifier(model.try_into()?))
			}
		}
	}
}

impl TryFrom<model::Regressor> for Regressor {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::Regressor) -> Result<Regressor> {
		let id = value.id;
		let columns = value
			.overall_column_stats
			.into_iter()
			.map(TryFrom::try_from)
			.collect::<Result<Vec<_>>>()?;
		match value.model {
			model::RegressionModel::Linear(inner_model) => {
				let feature_groups = inner_model
					.feature_groups
					.into_iter()
					.map(TryFrom::try_from)
					.collect::<Result<Vec<_>>>()?;
				Ok(Regressor {
					id,
					columns,
					feature_groups,
					model: RegressionModel::Linear(tangram_linear::Regressor {
						bias: inner_model.bias,
						weights: inner_model.weights.into(),
						means: inner_model.means,
					}),
				})
			}
			model::RegressionModel::Tree(inner_model) => {
				let feature_groups = inner_model
					.feature_groups
					.into_iter()
					.map(TryFrom::try_from)
					.collect::<Result<Vec<_>>>()?;
				Ok(Regressor {
					id,
					columns,
					feature_groups,
					model: RegressionModel::Tree(tangram_tree::Regressor {
						bias: inner_model.bias,
						trees: inner_model
							.trees
							.into_iter()
							.map(TryInto::try_into)
							.collect::<Result<Vec<_>>>()?,
					}),
				})
			}
		}
	}
}

impl TryFrom<model::BinaryClassifier> for BinaryClassifier {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::BinaryClassifier) -> Result<BinaryClassifier> {
		let id = value.id;
		let columns = value
			.overall_column_stats
			.into_iter()
			.map(TryFrom::try_from)
			.collect::<Result<Vec<_>>>()?;
		let negative_class = value.negative_class;
		let positive_class = value.positive_class;
		match value.model {
			model::BinaryClassificationModel::Linear(inner_model) => {
				let feature_groups = inner_model
					.feature_groups
					.into_iter()
					.map(TryFrom::try_from)
					.collect::<Result<Vec<_>>>()?;
				Ok(BinaryClassifier {
					id,
					columns,
					negative_class,
					positive_class,
					feature_groups,
					model: BinaryClassificationModel::Linear(tangram_linear::BinaryClassifier {
						weights: inner_model.weights.into(),
						bias: inner_model.bias,
						means: inner_model.means,
					}),
				})
			}
			model::BinaryClassificationModel::Tree(inner_model) => {
				let feature_groups = inner_model
					.feature_groups
					.into_iter()
					.map(TryFrom::try_from)
					.collect::<Result<Vec<_>>>()?;
				Ok(BinaryClassifier {
					id,
					columns,
					negative_class,
					positive_class,
					feature_groups,
					model: BinaryClassificationModel::Tree(tangram_tree::BinaryClassifier {
						bias: inner_model.bias,
						trees: inner_model
							.trees
							.into_iter()
							.map(TryInto::try_into)
							.collect::<Result<Vec<_>>>()?,
					}),
				})
			}
		}
	}
}

impl TryFrom<model::MulticlassClassifier> for MulticlassClassifier {
	type Error = tangram_util::error::Error;
	fn try_from(model: model::MulticlassClassifier) -> Result<MulticlassClassifier> {
		let id = model.id;
		let columns = model
			.overall_column_stats
			.into_iter()
			.map(TryFrom::try_from)
			.collect::<Result<Vec<_>>>()?;
		let classes = model.classes.clone();
		match model.model {
			model::MulticlassClassificationModel::Linear(inner_model) => {
				let n_classes = inner_model.n_classes.to_usize().unwrap();
				let n_features = inner_model.n_features.to_usize().unwrap();
				let weights =
					Array::from_shape_vec((n_features, n_classes), inner_model.weights).unwrap();
				let feature_groups = inner_model
					.feature_groups
					.into_iter()
					.map(TryFrom::try_from)
					.collect::<Result<Vec<_>>>()?;
				Ok(MulticlassClassifier {
					id,
					columns,
					classes,
					feature_groups,
					model: MulticlassClassificationModel::Linear(
						tangram_linear::MulticlassClassifier {
							weights,
							biases: inner_model.biases.into(),
							means: inner_model.means,
						},
					),
				})
			}
			model::MulticlassClassificationModel::Tree(inner_model) => {
				let feature_groups = inner_model
					.feature_groups
					.into_iter()
					.map(TryFrom::try_from)
					.collect::<Result<Vec<_>>>()?;
				Ok(MulticlassClassifier {
					id,
					columns,
					classes,
					feature_groups,
					model: MulticlassClassificationModel::Tree(
						tangram_tree::MulticlassClassifier {
							biases: inner_model.biases,
							trees: inner_model
								.trees
								.into_iter()
								.map(TryInto::try_into)
								.collect::<Result<Vec<_>>>()?,
							n_classes: inner_model.n_classes.to_usize().unwrap(),
							n_rounds: inner_model.n_rounds.to_usize().unwrap(),
						},
					),
				})
			}
		}
	}
}

impl TryFrom<model::ColumnStats> for Column {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::ColumnStats) -> Result<Column> {
		match value {
			model::ColumnStats::Unknown(value) => Ok(Column::Unknown(UnknownColumn {
				name: value.column_name,
			})),
			model::ColumnStats::Number(value) => Ok(Column::Number(NumberColumn {
				name: value.column_name,
			})),
			model::ColumnStats::Enum(value) => Ok(Column::Enum(EnumColumn {
				name: value.column_name,
				options: value.histogram.into_iter().map(|v| v.0).collect(),
			})),
			model::ColumnStats::Text(value) => Ok(Column::Text(TextColumn {
				name: value.column_name,
			})),
		}
	}
}

impl TryFrom<model::FeatureGroup> for features::FeatureGroup {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::FeatureGroup) -> Result<features::FeatureGroup> {
		match value {
			model::FeatureGroup::Identity(feature_group) => {
				Ok(features::FeatureGroup::Identity(feature_group.try_into()?))
			}
			model::FeatureGroup::Normalized(feature_group) => Ok(
				features::FeatureGroup::Normalized(feature_group.try_into()?),
			),
			model::FeatureGroup::OneHotEncoded(feature_group) => Ok(
				features::FeatureGroup::OneHotEncoded(feature_group.try_into()?),
			),
			model::FeatureGroup::BagOfWords(feature_group) => Ok(
				features::FeatureGroup::BagOfWords(feature_group.try_into()?),
			),
		}
	}
}

impl TryFrom<model::IdentityFeatureGroup> for features::IdentityFeatureGroup {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::IdentityFeatureGroup) -> Result<features::IdentityFeatureGroup> {
		Ok(features::IdentityFeatureGroup {
			source_column_name: value.source_column_name,
		})
	}
}

impl TryFrom<model::NormalizedFeatureGroup> for features::NormalizedFeatureGroup {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::NormalizedFeatureGroup) -> Result<features::NormalizedFeatureGroup> {
		Ok(features::NormalizedFeatureGroup {
			source_column_name: value.source_column_name,
			mean: value.mean,
			variance: value.variance,
		})
	}
}

impl TryFrom<model::OneHotEncodedFeatureGroup> for features::OneHotEncodedFeatureGroup {
	type Error = tangram_util::error::Error;
	fn try_from(
		value: model::OneHotEncodedFeatureGroup,
	) -> Result<features::OneHotEncodedFeatureGroup> {
		Ok(features::OneHotEncodedFeatureGroup {
			source_column_name: value.source_column_name,
			options: value.options,
		})
	}
}

impl TryFrom<model::BagOfWordsFeatureGroup> for features::BagOfWordsFeatureGroup {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::BagOfWordsFeatureGroup) -> Result<features::BagOfWordsFeatureGroup> {
		let tokens = value
			.tokens
			.into_iter()
			.map(|token| features::BagOfWordsFeatureGroupToken {
				token: token.token.try_into().unwrap(),
				idf: token.idf,
			})
			.collect::<Vec<_>>();
		let tokens_map = tokens
			.iter()
			.enumerate()
			.map(|(i, token)| (token.token.clone(), i))
			.collect();
		Ok(features::BagOfWordsFeatureGroup {
			source_column_name: value.source_column_name,
			tokenizer: value.tokenizer.try_into()?,
			tokens,
			tokens_map,
		})
	}
}

impl TryFrom<model::Token> for tangram_features::Token {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::Token) -> Result<tangram_features::Token> {
		match value {
			model::Token::Unigram(token) => Ok(tangram_features::Token::Unigram(token)),
			model::Token::Bigram(token_a, token_b) => {
				Ok(tangram_features::Token::Bigram(token_a, token_b))
			}
		}
	}
}

impl TryFrom<model::Tokenizer> for features::Tokenizer {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::Tokenizer) -> Result<features::Tokenizer> {
		match value {
			model::Tokenizer::Alphanumeric => Ok(features::Tokenizer::Alphanumeric),
		}
	}
}

impl TryFrom<model::Tree> for tangram_tree::Tree {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::Tree) -> Result<tangram_tree::Tree> {
		Ok(tangram_tree::Tree {
			nodes: value
				.nodes
				.into_iter()
				.map(TryInto::try_into)
				.collect::<Result<Vec<_>>>()?,
		})
	}
}

impl TryFrom<model::Node> for tangram_tree::Node {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::Node) -> Result<tangram_tree::Node> {
		match value {
			model::Node::Branch(value) => {
				Ok(tangram_tree::Node::Branch(tangram_tree::BranchNode {
					left_child_index: value.left_child_index.to_usize().unwrap(),
					right_child_index: value.right_child_index.to_usize().unwrap(),
					split: value.split.try_into()?,
					examples_fraction: value.examples_fraction,
				}))
			}
			model::Node::Leaf(value) => Ok(tangram_tree::Node::Leaf(tangram_tree::LeafNode {
				value: value.value,
				examples_fraction: value.examples_fraction,
			})),
		}
	}
}

impl TryFrom<model::BranchSplit> for tangram_tree::BranchSplit {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::BranchSplit) -> Result<tangram_tree::BranchSplit> {
		match value {
			model::BranchSplit::Continuous(value) => Ok(tangram_tree::BranchSplit::Continuous(
				tangram_tree::BranchSplitContinuous {
					feature_index: value.feature_index.to_usize().unwrap(),
					split_value: value.split_value,
					invalid_values_direction: if value.invalid_values_direction {
						tangram_tree::SplitDirection::Right
					} else {
						tangram_tree::SplitDirection::Left
					},
				},
			)),
			model::BranchSplit::Discrete(value) => Ok(tangram_tree::BranchSplit::Discrete(
				tangram_tree::BranchSplitDiscrete {
					feature_index: value.feature_index.to_usize().unwrap(),
					directions: value
						.directions
						.into_iter()
						.map(TryInto::try_into)
						.collect::<Result<_, _>>()?,
				},
			)),
		}
	}
}

impl TryFrom<model::SplitDirection> for tangram_tree::SplitDirection {
	type Error = tangram_util::error::Error;
	fn try_from(value: model::SplitDirection) -> Result<tangram_tree::SplitDirection> {
		Ok(match value {
			model::SplitDirection::Left => tangram_tree::SplitDirection::Left,
			model::SplitDirection::Right => tangram_tree::SplitDirection::Right,
		})
	}
}
