use crate::{features, model};
use anyhow::Result;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::{
	collections::BTreeMap,
	convert::{TryFrom, TryInto},
	num::NonZeroUsize,
};

#[derive(serde::Deserialize, Debug)]
pub struct PredictOptions {
	pub threshold: f32,
}

impl Default for PredictOptions {
	fn default() -> Self {
		Self { threshold: 0.5 }
	}
}

#[derive(serde::Deserialize, Debug)]
pub struct PredictInput(pub Vec<serde_json::Map<String, serde_json::Value>>);

#[derive(serde::Serialize, Debug)]
#[serde(untagged)]
pub enum PredictOutput {
	Regression(Vec<RegressionPredictOutput>),
	Classification(Vec<ClassificationPredictOutput>),
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegressionPredictOutput {
	pub value: f32,
	pub shap_output: Option<RegressionShapOutput>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegressionShapOutput {
	/// The baseline value is the value output by the model for this class before taking into account the feature values.
	pub baseline_value: f32,
	/// The output value will be the sum of the baseline value and the shap values of all features.
	pub output_value: f32,
	/// These are the shap values for each feature.
	pub feature_contributions: Vec<FeatureContribution>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationPredictOutput {
	pub class_name: String,
	pub probability: f32,
	pub probabilities: BTreeMap<String, f32>,
	pub shap_output: Option<ClassificationShapOutput>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationShapOutput {
	pub classes: BTreeMap<String, ClassificationShapOutputForClass>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationShapOutputForClass {
	/// The baseline value is the value output by the model for this class before taking into account the feature values.
	pub baseline_value: f32,
	/// The output value will be the sum of the baseline value and the shap values of all features.
	pub output_value: f32,
	/// These are the shap values for each feature.
	pub feature_contributions: Vec<FeatureContribution>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeatureContribution {
	/// This is a human readable name describing the feature.
	pub feature_name: String,
	/// This is the shap value for this feature.
	pub value: f32,
}

#[derive(Debug)]
pub enum PredictModel {
	LinearRegressor(LinearRegressorPredictModel),
	TreeRegressor(TreeRegressorPredictModel),
	LinearBinaryClassifier(LinearBinaryClassifierPredictModel),
	TreeBinaryClassifier(TreeBinaryClassifierPredictModel),
	TreeMulticlassClassifier(TreeMulticlassClassifierPredictModel),
	LinearMulticlassClassifier(LinearMulticlassClassifierPredictModel),
}

#[derive(Debug)]
pub struct LinearRegressorPredictModel {
	pub id: String,
	pub columns: Vec<Column>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: tangram_linear::Regressor,
}

#[derive(Debug)]
pub struct TreeRegressorPredictModel {
	pub id: String,
	pub columns: Vec<Column>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: tangram_tree::Regressor,
}

#[derive(Debug)]
pub struct LinearBinaryClassifierPredictModel {
	pub id: String,
	pub columns: Vec<Column>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: tangram_linear::BinaryClassifier,
}

#[derive(Debug)]
pub struct TreeBinaryClassifierPredictModel {
	pub id: String,
	pub columns: Vec<Column>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: tangram_tree::BinaryClassifier,
}

#[derive(Debug)]
pub struct LinearMulticlassClassifierPredictModel {
	pub id: String,
	pub columns: Vec<Column>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: tangram_linear::MulticlassClassifier,
}

#[derive(Debug)]
pub struct TreeMulticlassClassifierPredictModel {
	pub id: String,
	pub columns: Vec<Column>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: tangram_tree::MulticlassClassifier,
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
	model: &PredictModel,
	input: PredictInput,
	options: Option<PredictOptions>,
) -> PredictOutput {
	// Initialize the dataframe.
	let columns = match model {
		PredictModel::LinearRegressor(model) => model.columns.as_slice(),
		PredictModel::TreeRegressor(model) => model.columns.as_slice(),
		PredictModel::LinearBinaryClassifier(model) => model.columns.as_slice(),
		PredictModel::TreeBinaryClassifier(model) => model.columns.as_slice(),
		PredictModel::LinearMulticlassClassifier(model) => model.columns.as_slice(),
		PredictModel::TreeMulticlassClassifier(model) => model.columns.as_slice(),
	};
	let column_names = columns
		.iter()
		.map(|c| match c {
			Column::Unknown(c) => c.name.clone(),
			Column::Number(c) => c.name.clone(),
			Column::Enum(c) => c.name.clone(),
			Column::Text(c) => c.name.clone(),
		})
		.collect();
	let column_types = columns
		.iter()
		.map(|c| match c {
			Column::Unknown(_) => tangram_dataframe::ColumnType::Unknown,
			Column::Number(_) => tangram_dataframe::ColumnType::Number,
			Column::Enum(s) => tangram_dataframe::ColumnType::Enum {
				options: s.options.clone(),
			},
			Column::Text(_) => tangram_dataframe::ColumnType::Text,
		})
		.collect();
	let mut dataframe = tangram_dataframe::DataFrame::new(column_names, column_types);
	// Fill the dataframe with the input.
	for input in input.0 {
		for column in dataframe.columns.iter_mut() {
			match column {
				tangram_dataframe::Column::Unknown(column) => column.len += 1,
				tangram_dataframe::Column::Number(column) => {
					let value = match input.get(&column.name) {
						Some(serde_json::Value::Number(value)) => {
							value.as_f64().unwrap().to_f32().unwrap()
						}
						_ => std::f32::NAN,
					};
					column.data.push(value);
				}
				tangram_dataframe::Column::Enum(column) => {
					let value = input.get(&column.name).and_then(|value| value.as_str());
					let value = value.and_then(|value| {
						column
							.options
							.iter()
							.position(|option| option == value)
							.map(|position| NonZeroUsize::new(position + 1).unwrap())
					});
					column.data.push(value);
				}
				tangram_dataframe::Column::Text(column) => {
					let value = input
						.get(&column.name)
						.and_then(|value| value.as_str())
						.unwrap_or("")
						.to_owned();
					column.data.push(value);
				}
			}
		}
	}
	// Make the predictions by matching on the model type.
	match model {
		PredictModel::LinearRegressor(model) => {
			let n_examples = dataframe.nrows();
			let n_features = model.feature_groups.iter().map(|f| f.n_features()).sum();
			let mut features = unsafe { Array2::uninitialized((n_examples, n_features)) };
			let mut predictions = unsafe { Array1::uninitialized(n_examples) };
			features::compute_features_ndarray(
				&dataframe.view(),
				&model.feature_groups,
				features.view_mut(),
				&|| {},
			);
			model.model.predict(features.view(), predictions.view_mut());
			let feature_names = compute_feature_names(&model.feature_groups);
			let shap_values = model.model.compute_shap_values(features.view());
			let output = predictions
				.into_iter()
				.zip(shap_values.into_iter())
				.map(|(prediction, shap_values)| {
					let feature_contributions = feature_names
						.iter()
						.zip(shap_values.feature_contributions.iter())
						.map(|(feature_name, value)| FeatureContribution {
							feature_name: feature_name.clone(),
							value: *value,
						})
						.collect();
					let shap_output = RegressionShapOutput {
						baseline_value: shap_values.baseline_value,
						output_value: shap_values.output_value,
						feature_contributions,
					};
					RegressionPredictOutput {
						value: *prediction,
						shap_output: Some(shap_output),
					}
				})
				.collect();
			PredictOutput::Regression(output)
		}
		PredictModel::TreeRegressor(model) => {
			let n_examples = dataframe.nrows();
			let n_features = model
				.feature_groups
				.iter()
				.map(|g| g.n_features())
				.sum::<usize>();
			let mut features = unsafe { Array2::uninitialized((dataframe.nrows(), n_features)) };
			features::compute_features_ndarray_value(
				&dataframe.view(),
				&model.feature_groups,
				features.view_mut(),
				&|| {},
			);
			let mut predictions = unsafe { Array1::uninitialized(n_examples) };
			model.model.predict(features.view(), predictions.view_mut());
			let feature_names = compute_feature_names(&model.feature_groups);
			let shap_values = model.model.compute_shap_values(features.view());
			let output = predictions
				.into_iter()
				.zip(shap_values.into_iter())
				.map(|(prediction, shap_values)| {
					let feature_contributions = feature_names
						.iter()
						.zip(shap_values.feature_contributions.iter())
						.map(|(feature_name, value)| FeatureContribution {
							feature_name: feature_name.clone(),
							value: *value,
						})
						.collect();
					let shap_output = RegressionShapOutput {
						baseline_value: shap_values.baseline_value,
						output_value: shap_values.output_value,
						feature_contributions,
					};
					RegressionPredictOutput {
						value: *prediction,
						shap_output: Some(shap_output),
					}
				})
				.collect();
			PredictOutput::Regression(output)
		}
		PredictModel::LinearBinaryClassifier(model) => {
			let n_examples = dataframe.nrows();
			let n_features = model.feature_groups.iter().map(|f| f.n_features()).sum();
			let mut features = unsafe { Array2::uninitialized((n_examples, n_features)) };
			let mut probabilities = unsafe { Array2::uninitialized((n_examples, 2)) };
			features::compute_features_ndarray(
				&dataframe.view(),
				&model.feature_groups,
				features.view_mut(),
				&|| {},
			);
			model
				.model
				.predict(features.view(), probabilities.view_mut());
			let feature_names = compute_feature_names(&model.feature_groups);
			let shap_values = model.model.compute_shap_values(features.view());
			let threshold = match options {
				Some(options) => options.threshold,
				None => 0.5,
			};
			let output = probabilities
				.axis_iter(Axis(0))
				.zip(shap_values.iter())
				.map(|(probabilities, shap_values)| {
					let (probability, class_name) = if probabilities[1] >= threshold {
						(probabilities[1], model.model.classes[1].clone())
					} else {
						(probabilities[0], model.model.classes[0].clone())
					};
					let probabilities = probabilities
						.iter()
						.zip(model.model.classes.iter())
						.map(|(p, c)| (c.clone(), *p))
						.collect::<BTreeMap<String, f32>>();
					let feature_contributions = feature_names
						.iter()
						.zip(shap_values.feature_contributions.iter())
						.map(|(feature_name, value)| FeatureContribution {
							feature_name: feature_name.clone(),
							value: *value,
						})
						.collect();
					let mut classes = BTreeMap::new();
					classes.insert(
						model.model.classes[1].clone(),
						ClassificationShapOutputForClass {
							baseline_value: shap_values.baseline_value,
							output_value: shap_values.output_value,
							feature_contributions,
						},
					);
					let shap_output = ClassificationShapOutput { classes };
					ClassificationPredictOutput {
						class_name,
						probability,
						probabilities,
						shap_output: Some(shap_output),
					}
				})
				.collect();
			PredictOutput::Classification(output)
		}
		PredictModel::TreeBinaryClassifier(model) => {
			let n_examples = dataframe.nrows();
			let n_features = model
				.feature_groups
				.iter()
				.map(|g| g.n_features())
				.sum::<usize>();
			let mut features = unsafe { Array2::uninitialized((dataframe.nrows(), n_features)) };
			features::compute_features_ndarray_value(
				&dataframe.view(),
				&model.feature_groups,
				features.view_mut(),
				&|| {},
			);
			let mut probabilities = unsafe { Array2::uninitialized((n_examples, 2)) };
			model
				.model
				.predict(features.view(), probabilities.view_mut());
			let feature_names = compute_feature_names(&model.feature_groups);
			let shap_values = model.model.compute_shap_values(features.view());
			let threshold = match options {
				Some(options) => options.threshold,
				None => 0.5,
			};
			let output = probabilities
				.axis_iter(Axis(0))
				.zip(shap_values.iter())
				.map(|(probabilities, shap_values)| {
					let (probability, class_name) = if probabilities[1] >= threshold {
						(probabilities[1], model.model.classes[1].clone())
					} else {
						(probabilities[0], model.model.classes[0].clone())
					};
					let probabilities = probabilities
						.iter()
						.zip(model.model.classes.iter())
						.map(|(p, c)| (c.clone(), *p))
						.collect::<BTreeMap<String, f32>>();
					let feature_contributions = feature_names
						.iter()
						.zip(shap_values.feature_contributions.iter())
						.map(|(feature_name, value)| FeatureContribution {
							feature_name: feature_name.clone(),
							value: *value,
						})
						.collect();
					let mut classes = BTreeMap::new();
					classes.insert(
						model.model.classes[1].clone(),
						ClassificationShapOutputForClass {
							baseline_value: shap_values.baseline_value,
							output_value: shap_values.output_value,
							feature_contributions,
						},
					);
					let shap_output = ClassificationShapOutput { classes };
					ClassificationPredictOutput {
						class_name,
						probability,
						probabilities,
						shap_output: Some(shap_output),
					}
				})
				.collect();
			PredictOutput::Classification(output)
		}
		PredictModel::LinearMulticlassClassifier(model) => {
			let n_examples = dataframe.nrows();
			let n_classes = model.model.classes.len();
			let n_features = model.feature_groups.iter().map(|f| f.n_features()).sum();
			let mut features = unsafe { Array2::uninitialized((n_examples, n_features)) };
			let mut probabilities = unsafe { Array2::uninitialized((n_examples, n_classes)) };
			features::compute_features_ndarray(
				&dataframe.view(),
				&model.feature_groups,
				features.view_mut(),
				&|| {},
			);
			model
				.model
				.predict(features.view(), probabilities.view_mut());
			let output = probabilities
				.axis_iter(Axis(0))
				.map(|probabilities| {
					let (probability, class_name) = probabilities
						.iter()
						.zip(model.model.classes.iter())
						.max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
						.unwrap();
					let probabilities = probabilities
						.iter()
						.zip(model.model.classes.iter())
						.map(|(p, c)| (c.clone(), *p))
						.collect::<BTreeMap<String, f32>>();
					ClassificationPredictOutput {
						class_name: class_name.to_owned(),
						probability: *probability,
						probabilities,
						shap_output: None,
					}
				})
				.collect();
			PredictOutput::Classification(output)
		}
		PredictModel::TreeMulticlassClassifier(model) => {
			let n_examples = dataframe.nrows();
			let n_classes = model.model.classes.len();
			let n_features = model
				.feature_groups
				.iter()
				.map(|g| g.n_features())
				.sum::<usize>();
			let mut features = unsafe { Array2::uninitialized((dataframe.nrows(), n_features)) };
			features::compute_features_ndarray_value(
				&dataframe.view(),
				&model.feature_groups,
				features.view_mut(),
				&|| {},
			);
			let mut probabilities = unsafe { Array2::uninitialized((n_examples, n_classes)) };
			model
				.model
				.predict(features.view(), probabilities.view_mut());
			let output = probabilities
				.axis_iter(Axis(0))
				.map(|probabilities| {
					let (probability, class_name) = probabilities
						.iter()
						.zip(model.model.classes.iter())
						.max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
						.unwrap();
					let probabilities = probabilities
						.iter()
						.zip(model.model.classes.iter())
						.map(|(p, c)| (c.clone(), *p))
						.collect::<BTreeMap<String, f32>>();
					ClassificationPredictOutput {
						class_name: class_name.to_owned(),
						probability: *probability,
						probabilities,
						shap_output: None,
					}
				})
				.collect();
			PredictOutput::Classification(output)
		}
	}
}

fn compute_feature_names(feature_groups: &[features::FeatureGroup]) -> Vec<String> {
	feature_groups
		.iter()
		.flat_map(|feature_group| match feature_group {
			features::FeatureGroup::Identity(identity_feature_group) => {
				vec![identity_feature_group.source_column_name.to_owned()]
			}
			features::FeatureGroup::Normalized(normalized_feature_group) => {
				vec![normalized_feature_group.source_column_name.to_owned()]
			}
			features::FeatureGroup::OneHotEncoded(one_hot_encoded_feature_group) => {
				one_hot_encoded_feature_group
					.categories
					.iter()
					.map(|category| {
						format!(
							"{} = \"{}\"",
							one_hot_encoded_feature_group.source_column_name, category
						)
					})
					.collect()
			}
			features::FeatureGroup::BagOfWords(bag_of_words_feature_group) => {
				bag_of_words_feature_group
					.tokens
					.iter()
					.map(|(token, _)| {
						format!(
							"{} contains \"{}\"",
							bag_of_words_feature_group.source_column_name, token
						)
					})
					.collect()
			}
		})
		.collect()
}

impl TryFrom<model::Model> for PredictModel {
	type Error = anyhow::Error;
	fn try_from(value: model::Model) -> Result<Self> {
		match value {
			model::Model::Regressor(model) => {
				let id = model.id;
				let columns = model
					.overall_column_stats
					.into_iter()
					.map(TryFrom::try_from)
					.collect::<Result<Vec<_>>>()?;
				match model.model {
					model::RegressionModel::Linear(model) => {
						let feature_groups = model
							.feature_groups
							.into_iter()
							.map(TryFrom::try_from)
							.collect::<Result<Vec<_>>>()?;
						Ok(Self::LinearRegressor(LinearRegressorPredictModel {
							id,
							columns,
							feature_groups,
							model: tangram_linear::Regressor {
								bias: model.bias,
								weights: model.weights.into(),
								means: model.means,
								losses: model.losses,
							},
						}))
					}
					model::RegressionModel::Tree(model) => {
						let feature_groups = model
							.feature_groups
							.into_iter()
							.map(TryFrom::try_from)
							.collect::<Result<Vec<_>>>()?;
						Ok(Self::TreeRegressor(TreeRegressorPredictModel {
							id,
							columns,
							feature_groups,
							model: tangram_tree::Regressor {
								bias: model.bias,
								trees: model
									.trees
									.into_iter()
									.map(TryInto::try_into)
									.collect::<Result<Vec<_>>>()?,
								feature_importances: Some(model.feature_importances),
								losses: Some(model.losses),
							},
						}))
					}
				}
			}
			model::Model::Classifier(model) => {
				let id = model.id;
				let columns = model
					.overall_column_stats
					.into_iter()
					.map(TryFrom::try_from)
					.collect::<Result<Vec<_>>>()?;
				match model.model {
					model::ClassificationModel::LinearBinary(model) => {
						let feature_groups = model
							.feature_groups
							.into_iter()
							.map(TryFrom::try_from)
							.collect::<Result<Vec<_>>>()?;
						Ok(Self::LinearBinaryClassifier(
							LinearBinaryClassifierPredictModel {
								id,
								columns,
								feature_groups,
								model: tangram_linear::BinaryClassifier {
									weights: model.weights.into(),
									bias: model.bias,
									means: model.means,
									losses: model.losses,
									classes: model.classes,
								},
							},
						))
					}
					model::ClassificationModel::TreeBinary(model) => {
						let feature_groups = model
							.feature_groups
							.into_iter()
							.map(TryFrom::try_from)
							.collect::<Result<Vec<_>>>()?;
						Ok(Self::TreeBinaryClassifier(
							TreeBinaryClassifierPredictModel {
								id,
								columns,
								feature_groups,
								model: tangram_tree::BinaryClassifier {
									bias: model.bias,
									trees: model
										.trees
										.into_iter()
										.map(TryInto::try_into)
										.collect::<Result<Vec<_>>>()?,
									feature_importances: Some(model.feature_importances),
									losses: model.losses,
									classes: model.classes,
								},
							},
						))
					}
					model::ClassificationModel::LinearMulticlass(model) => {
						let n_classes = model.n_classes.to_usize().unwrap();
						let n_features = model.n_features.to_usize().unwrap();
						let weights =
							Array2::from_shape_vec((n_features, n_classes), model.weights).unwrap();
						let feature_groups = model
							.feature_groups
							.into_iter()
							.map(TryFrom::try_from)
							.collect::<Result<Vec<_>>>()?;
						Ok(Self::LinearMulticlassClassifier(
							LinearMulticlassClassifierPredictModel {
								id,
								columns,
								feature_groups,
								model: tangram_linear::MulticlassClassifier {
									weights,
									biases: model.biases.into(),
									means: model.means,
									losses: model.losses,
									classes: model.classes,
								},
							},
						))
					}
					model::ClassificationModel::TreeMulticlass(model) => {
						let feature_groups = model
							.feature_groups
							.into_iter()
							.map(TryFrom::try_from)
							.collect::<Result<Vec<_>>>()?;
						Ok(Self::TreeMulticlassClassifier(
							TreeMulticlassClassifierPredictModel {
								id,
								columns,
								feature_groups,
								model: tangram_tree::MulticlassClassifier {
									biases: model.biases,
									trees: model
										.trees
										.into_iter()
										.map(TryInto::try_into)
										.collect::<Result<Vec<_>>>()?,
									feature_importances: Some(model.feature_importances),
									losses: model.losses,
									classes: model.classes,
									n_classes: model.n_classes.to_usize().unwrap(),
									n_rounds: model.n_rounds.to_usize().unwrap(),
								},
							},
						))
					}
				}
			}
		}
	}
}

impl TryFrom<model::Tree> for tangram_tree::Tree {
	type Error = anyhow::Error;
	fn try_from(value: model::Tree) -> Result<Self> {
		Ok(Self {
			nodes: value
				.nodes
				.into_iter()
				.map(TryInto::try_into)
				.collect::<Result<Vec<_>>>()?,
		})
	}
}

impl TryFrom<model::Node> for tangram_tree::Node {
	type Error = anyhow::Error;
	fn try_from(value: model::Node) -> Result<Self> {
		match value {
			model::Node::Branch(value) => Ok(Self::Branch(tangram_tree::BranchNode {
				left_child_index: value.left_child_index.to_usize().unwrap(),
				right_child_index: value.right_child_index.to_usize().unwrap(),
				split: value.split.try_into()?,
				examples_fraction: value.examples_fraction,
			})),
			model::Node::Leaf(value) => Ok(Self::Leaf(tangram_tree::LeafNode {
				value: value.value,
				examples_fraction: value.examples_fraction,
			})),
		}
	}
}

impl TryFrom<model::BranchSplit> for tangram_tree::BranchSplit {
	type Error = anyhow::Error;
	fn try_from(value: model::BranchSplit) -> Result<Self> {
		match value {
			model::BranchSplit::Continuous(s) => {
				Ok(Self::Continuous(tangram_tree::BranchSplitContinuous {
					feature_index: s.feature_index.to_usize().unwrap(),
					split_value: s.split_value,
					invalid_values_direction: if s.invalid_values_direction {
						tangram_tree::SplitDirection::Right
					} else {
						tangram_tree::SplitDirection::Left
					},
				}))
			}
			model::BranchSplit::Discrete(s) => {
				Ok(Self::Discrete(tangram_tree::BranchSplitDiscrete {
					feature_index: s.feature_index.to_usize().unwrap(),
					directions: s
						.directions
						.into_iter()
						.map(TryInto::try_into)
						.collect::<Result<_, _>>()?,
				}))
			}
		}
	}
}

impl TryFrom<model::SplitDirection> for tangram_tree::SplitDirection {
	type Error = anyhow::Error;
	fn try_from(value: model::SplitDirection) -> Result<Self> {
		Ok(match value {
			model::SplitDirection::Left => Self::Left,
			model::SplitDirection::Right => Self::Right,
		})
	}
}

impl TryFrom<model::ColumnStats> for Column {
	type Error = anyhow::Error;
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
	type Error = anyhow::Error;
	fn try_from(value: model::FeatureGroup) -> Result<Self> {
		match value {
			model::FeatureGroup::Identity(f) => Ok(features::FeatureGroup::Identity(
				features::IdentityFeatureGroup {
					source_column_name: f.source_column_name,
				},
			)),
			model::FeatureGroup::Normalized(f) => Ok(features::FeatureGroup::Normalized(
				features::NormalizedFeatureGroup {
					source_column_name: f.source_column_name,
					mean: f.mean,
					variance: f.variance,
				},
			)),
			model::FeatureGroup::OneHotEncoded(f) => Ok(features::FeatureGroup::OneHotEncoded(
				features::OneHotEncodedFeatureGroup {
					source_column_name: f.source_column_name,
					categories: f.categories,
				},
			)),
			model::FeatureGroup::BagOfWords(f) => Ok(features::FeatureGroup::BagOfWords(
				features::BagOfWordsFeatureGroup {
					source_column_name: f.source_column_name,
					tokenizer: f.tokenizer.try_into()?,
					tokens: f.tokens,
				},
			)),
		}
	}
}

impl TryFrom<model::Tokenizer> for features::Tokenizer {
	type Error = anyhow::Error;
	fn try_from(value: model::Tokenizer) -> Result<features::Tokenizer> {
		match value {
			model::Tokenizer::Alphanumeric => Ok(features::Tokenizer::Alphanumeric),
		}
	}
}
