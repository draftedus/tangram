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
	pub shap_values: Option<ShapValues>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationPredictOutput {
	pub class_name: String,
	pub probabilities: BTreeMap<String, f32>,
	pub shap_values: Option<BTreeMap<String, ShapValues>>,
}

#[derive(serde::Serialize, Debug)]
pub struct ShapValues {
	pub baseline: f32,
	pub values: Vec<(String, f32)>,
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

impl Column {
	fn column_name(&self) -> String {
		match self {
			Self::Unknown(c) => c.name.clone(),
			Self::Number(c) => c.name.clone(),
			Self::Enum(c) => c.name.clone(),
			Self::Text(c) => c.name.clone(),
		}
	}
}

pub fn predict(
	model: &PredictModel,
	input: PredictInput,
	options: Option<PredictOptions>,
) -> PredictOutput {
	// initialize the dataframe
	let columns = match model {
		PredictModel::LinearRegressor(model) => model.columns.as_slice(),
		PredictModel::TreeRegressor(model) => model.columns.as_slice(),
		PredictModel::LinearBinaryClassifier(model) => model.columns.as_slice(),
		PredictModel::TreeBinaryClassifier(model) => model.columns.as_slice(),
		PredictModel::LinearMulticlassClassifier(model) => model.columns.as_slice(),
		PredictModel::TreeMulticlassClassifier(model) => model.columns.as_slice(),
	};
	let column_names = columns.iter().map(|c| c.column_name()).collect();
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

	// fill the dataframe with the input
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

	match model {
		PredictModel::LinearRegressor(model) => {
			let n_examples = dataframe.nrows();
			let n_features = model.feature_groups.iter().map(|f| f.n_features()).sum();
			let feature_groups = &model.feature_groups;
			let model = &model.model;
			let mut features = unsafe { Array2::uninitialized((n_examples, n_features)) };
			let mut shap_values = Array3::zeros((features.nrows(), 1, features.ncols() + 1));
			let mut predictions = unsafe { Array1::uninitialized(n_examples) };
			features::compute_features_ndarray(
				&dataframe.view(),
				feature_groups,
				features.view_mut(),
				&|| {},
			);
			model.predict(features.view(), predictions.view_mut());
			model.compute_shap_values(features.view(), shap_values.view_mut());
			let shap_values = compute_shap_values_regression_output_linear(
				feature_groups.as_slice(),
				shap_values.view(),
				features.view(),
				dataframe.view(),
			);
			let output = predictions
				.into_iter()
				.zip(shap_values.into_iter())
				.map(|(prediction, shap_values)| RegressionPredictOutput {
					value: *prediction,
					shap_values: Some(shap_values),
				})
				.collect();
			PredictOutput::Regression(output)
		}
		PredictModel::TreeRegressor(model) => {
			let n_examples = dataframe.nrows();
			let feature_groups = &model.feature_groups;
			let model = &model.model;
			let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
			let mut features = unsafe { Array2::uninitialized((dataframe.nrows(), n_features)) };
			let mut shap_values = Array3::zeros((features.nrows(), 1, features.ncols() + 1));
			features::compute_features_ndarray_value(
				&dataframe.view(),
				feature_groups,
				features.view_mut(),
				&|| {},
			);
			let mut predictions = unsafe { Array1::uninitialized(n_examples) };
			model.predict(features.view(), predictions.view_mut());
			model.compute_shap_values(features.view(), shap_values.view_mut());
			let shap_values = compute_shap_values_regression_output_tree(
				feature_groups.as_slice(),
				shap_values.view(),
				features.view(),
				dataframe.view(),
			);
			let output = predictions
				.into_iter()
				.zip(shap_values.into_iter())
				.map(|(prediction, shap_values)| RegressionPredictOutput {
					value: *prediction,
					shap_values: Some(shap_values),
				})
				.collect();
			PredictOutput::Regression(output)
		}
		PredictModel::LinearBinaryClassifier(model) => {
			let n_examples = dataframe.nrows();
			let n_features = model.feature_groups.iter().map(|f| f.n_features()).sum();
			let feature_groups = &model.feature_groups;
			let model = &model.model;
			let mut features = unsafe { Array2::uninitialized((n_examples, n_features)) };
			let mut shap_values = Array3::zeros((features.nrows(), 1, features.ncols() + 1));
			let mut probabilities = unsafe { Array2::uninitialized((n_examples, 2)) };
			features::compute_features_ndarray(
				&dataframe.view(),
				feature_groups,
				features.view_mut(),
				&|| {},
			);
			model.predict(features.view(), probabilities.view_mut());
			model.compute_shap_values(features.view(), shap_values.view_mut());
			let shap_values = compute_shap_values_classification_output_linear(
				feature_groups.as_slice(),
				&[model.classes[1].clone()],
				shap_values.view(),
				features.view(),
				dataframe.view(),
			);
			let threshold = match options {
				Some(options) => options.threshold,
				None => 0.5,
			};
			let output = probabilities
				.genrows()
				.into_iter()
				.zip(shap_values.into_iter())
				.map(|(probabilities, shap_values)| {
					let class_name = if probabilities[1] >= threshold {
						model.classes[1].clone()
					} else {
						model.classes[0].clone()
					};
					let probabilities = probabilities
						.iter()
						.zip(model.classes.iter())
						.map(|(p, c)| (c.clone(), *p))
						.collect::<BTreeMap<String, f32>>();
					ClassificationPredictOutput {
						class_name,
						probabilities,
						shap_values: Some(shap_values),
					}
				})
				.collect();
			PredictOutput::Classification(output)
		}
		PredictModel::TreeBinaryClassifier(model) => {
			let n_examples = dataframe.nrows();
			let feature_groups = &model.feature_groups;
			let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
			let model = &model.model;
			let mut features = unsafe { Array2::uninitialized((dataframe.nrows(), n_features)) };
			features::compute_features_ndarray_value(
				&dataframe.view(),
				feature_groups,
				features.view_mut(),
				&|| {},
			);
			let mut probabilities = unsafe { Array2::uninitialized((n_examples, 2)) };
			let mut shap_values = Array3::zeros((features.nrows(), 1, features.ncols() + 1));
			model.predict(features.view(), probabilities.view_mut());
			model.compute_shap_values(features.view(), shap_values.view_mut());
			let shap_values = compute_shap_values_classification_output_tree(
				feature_groups.as_slice(),
				&[model.classes[1].clone()],
				shap_values.view(),
				features.view(),
				dataframe.view(),
			);
			let threshold = match options {
				Some(options) => options.threshold,
				None => 0.5,
			};
			let output = probabilities
				.genrows()
				.into_iter()
				.zip(shap_values.into_iter())
				.map(|(probabilities, shap_values)| {
					let class_name = if probabilities[1] >= threshold {
						model.classes[1].clone()
					} else {
						model.classes[0].clone()
					};
					let probabilities = probabilities
						.iter()
						.zip(model.classes.iter())
						.map(|(p, c)| (c.clone(), *p))
						.collect::<BTreeMap<String, f32>>();
					ClassificationPredictOutput {
						class_name,
						probabilities,
						shap_values: Some(shap_values),
					}
				})
				.collect();
			PredictOutput::Classification(output)
		}
		PredictModel::LinearMulticlassClassifier(model) => {
			let n_examples = dataframe.nrows();
			let n_classes = model.model.classes.len();
			let n_features = model.feature_groups.iter().map(|f| f.n_features()).sum();
			let feature_groups = &model.feature_groups;
			let model = &model.model;
			let mut features = unsafe { Array2::uninitialized((n_examples, n_features)) };
			let mut shap_values =
				Array3::zeros((features.nrows(), n_classes, features.ncols() + 1));
			let mut probabilities = unsafe { Array2::uninitialized((n_examples, n_classes)) };
			features::compute_features_ndarray(
				&dataframe.view(),
				feature_groups,
				features.view_mut(),
				&|| {},
			);
			model.predict(features.view(), probabilities.view_mut());
			model.compute_shap_values(features.view(), shap_values.view_mut());
			let shap_values = compute_shap_values_classification_output_linear(
				feature_groups.as_slice(),
				&model.classes,
				shap_values.view(),
				features.view(),
				dataframe.view(),
			);
			let output = probabilities
				.genrows()
				.into_iter()
				.zip(shap_values.into_iter())
				.map(|(probabilities, shap_values)| {
					let class_name = probabilities
						.iter()
						.zip(model.classes.iter())
						.max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
						.unwrap()
						.1
						.clone();
					let probabilities = probabilities
						.iter()
						.zip(model.classes.iter())
						.map(|(p, c)| (c.clone(), *p))
						.collect::<BTreeMap<String, f32>>();
					ClassificationPredictOutput {
						class_name,
						probabilities,
						shap_values: Some(shap_values),
					}
				})
				.collect();
			PredictOutput::Classification(output)
		}
		PredictModel::TreeMulticlassClassifier(model) => {
			let n_examples = dataframe.nrows();
			let feature_groups = &model.feature_groups;
			let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
			let model = &model.model;
			let n_classes = model.classes.len();
			let mut features = unsafe { Array2::uninitialized((dataframe.nrows(), n_features)) };
			features::compute_features_ndarray_value(
				&dataframe.view(),
				feature_groups,
				features.view_mut(),
				&|| {},
			);
			let mut probabilities = unsafe { Array2::uninitialized((n_examples, n_classes)) };
			let mut shap_values =
				Array3::zeros((features.nrows(), n_classes, features.ncols() + 1));
			model.predict(features.view(), probabilities.view_mut());
			model.compute_shap_values(features.view(), shap_values.view_mut());
			let shap_values = compute_shap_values_classification_output_tree(
				feature_groups.as_slice(),
				&model.classes.as_slice(),
				shap_values.view(),
				features.view(),
				dataframe.view(),
			);
			let output = probabilities
				.genrows()
				.into_iter()
				.zip(shap_values.into_iter())
				.map(|(probabilities, shap_values)| {
					let class_index = probabilities
						.iter()
						.enumerate()
						.max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
						.unwrap()
						.0;
					let class_name = model.classes[class_index].clone();
					let probabilities = probabilities
						.iter()
						.zip(model.classes.iter())
						.map(|(p, c)| (c.clone(), *p))
						.collect::<BTreeMap<String, f32>>();
					ClassificationPredictOutput {
						class_name,
						probabilities,
						shap_values: Some(shap_values),
					}
				})
				.collect();
			PredictOutput::Classification(output)
		}
	}
}

fn compute_shap_values_regression_output_linear(
	feature_groups: &[features::FeatureGroup],
	shap_values: ArrayView3<f32>,
	features: ArrayView2<f32>,
	dataframe: tangram_dataframe::DataFrameView,
) -> Vec<ShapValues> {
	let feature_names = compute_feature_names(feature_groups);
	let feature_group_map = compute_feature_group_map(feature_groups);
	shap_values
		.axis_iter(Axis(0))
		.zip(features.axis_iter(Axis(0)))
		.map(|(shap_values, features)| {
			let baseline = shap_values.get([0, shap_values.len() - 1]).unwrap();
			let mut shap_values = compute_shap_values_linear(
				feature_group_map.as_slice(),
				feature_groups,
				feature_names.as_slice(),
				features.as_slice().unwrap(),
				shap_values.row(0).as_slice().unwrap(),
				&dataframe,
			);
			shap_values.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
			ShapValues {
				baseline: *baseline,
				values: shap_values,
			}
		})
		.collect()
}

fn compute_shap_values_regression_output_tree(
	feature_groups: &[features::FeatureGroup],
	shap_values: ArrayView3<f32>,
	features: ArrayView2<tangram_dataframe::Value>,
	dataframe: tangram_dataframe::DataFrameView,
) -> Vec<ShapValues> {
	let feature_names = compute_feature_names(feature_groups);
	let feature_group_map = compute_feature_group_map(feature_groups);
	shap_values
		.axis_iter(Axis(0))
		.zip(features.axis_iter(Axis(0)))
		.map(|(shap_values, features)| {
			let baseline = shap_values.get([0, shap_values.len() - 1]).unwrap();
			let mut shap_values = compute_shap_values_tree(
				feature_group_map.as_slice(),
				feature_groups,
				feature_names.as_slice(),
				features,
				shap_values.row(0),
				&dataframe,
			);
			shap_values.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
			ShapValues {
				baseline: *baseline,
				values: shap_values,
			}
		})
		.collect()
}

fn compute_shap_values_classification_output_linear(
	feature_groups: &[features::FeatureGroup],
	classes: &[String],
	shap_values: ArrayView3<f32>,
	features: ArrayView2<f32>,
	dataframe: tangram_dataframe::DataFrameView,
) -> Vec<BTreeMap<String, ShapValues>> {
	let feature_names = compute_feature_names(feature_groups);
	let feature_group_map = compute_feature_group_map(feature_groups);
	shap_values
		.axis_iter(Axis(0))
		.zip(features.axis_iter(Axis(0)))
		.map(|(shap_values, features)| {
			classes
				.iter()
				.enumerate()
				.map(|(class_index, class)| {
					let shap_values = shap_values.slice(s![class_index, ..]);
					let baseline = shap_values[shap_values.len() - 1];
					let mut shap_values = compute_shap_values_linear(
						feature_group_map.as_slice(),
						feature_groups,
						feature_names.as_slice(),
						features.as_slice().unwrap(),
						shap_values.as_slice().unwrap(),
						&dataframe,
					);
					shap_values.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
					(
						class.clone(),
						ShapValues {
							baseline,
							values: shap_values,
						},
					)
				})
				.collect::<BTreeMap<String, ShapValues>>()
		})
		.collect()
}

fn compute_shap_values_linear(
	feature_group_map: &[usize],
	feature_groups: &[features::FeatureGroup],
	feature_names: &[String],
	features: &[f32],
	shap_values: &[f32],
	_dataframe: &tangram_dataframe::DataFrameView,
) -> Vec<(String, f32)> {
	feature_names
		.iter()
		.zip(shap_values.iter())
		.zip(features)
		.zip(feature_group_map.iter())
		.map(|(((name, value), feature), feature_group_index)| {
			let feature_str = match &feature_groups[*feature_group_index] {
				features::FeatureGroup::BagOfWords(_) => {
					if feature.partial_cmp(&0.0) == Some(std::cmp::Ordering::Equal) {
						"does not contain".to_owned()
					} else {
						"contains".to_owned()
					}
				}
				features::FeatureGroup::Normalized(_) => "".to_owned(),
				features::FeatureGroup::OneHotEncoded(_) => {
					if feature.partial_cmp(&0.0) == Some(std::cmp::Ordering::Equal) {
						"false".to_owned()
					} else {
						"true".to_owned()
					}
				}
				features::FeatureGroup::Identity(_) => feature.to_string(),
			};
			(name.replace("{}", feature_str.as_str()), *value)
		})
		.collect::<Vec<(String, f32)>>()
}

fn compute_shap_values_classification_output_tree(
	feature_groups: &[features::FeatureGroup],
	classes: &[String],
	shap_values: ArrayView3<f32>,
	features: ArrayView2<tangram_dataframe::Value>,
	dataframe: tangram_dataframe::DataFrameView,
) -> Vec<BTreeMap<String, ShapValues>> {
	let feature_names = compute_feature_names(feature_groups);
	let feature_group_map = compute_feature_group_map(feature_groups);
	shap_values
		.axis_iter(Axis(0))
		.zip(features.axis_iter(Axis(0)))
		.map(|(shap_values, features)| {
			classes
				.iter()
				.enumerate()
				.map(|(class_index, class)| {
					let shap_values = shap_values.slice(s![class_index, ..]);
					let baseline = shap_values[shap_values.len() - 1];
					let mut shap_values = compute_shap_values_tree(
						feature_group_map.as_slice(),
						feature_groups,
						feature_names.as_slice(),
						features,
						shap_values,
						&dataframe,
					);
					shap_values.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
					(
						class.clone(),
						ShapValues {
							baseline,
							values: shap_values,
						},
					)
				})
				.collect::<BTreeMap<String, ShapValues>>()
		})
		.collect()
}

fn compute_shap_values_tree(
	feature_group_map: &[usize],
	feature_groups: &[features::FeatureGroup],
	feature_names: &[String],
	features: ArrayView1<tangram_dataframe::Value>,
	shap_values: ArrayView1<f32>,
	dataframe: &tangram_dataframe::DataFrameView,
) -> Vec<(String, f32)> {
	feature_names
		.iter()
		.zip(shap_values.iter())
		.zip(features)
		.zip(feature_group_map.iter())
		.map(|(((name, value), feature), feature_group_index)| {
			let feature = match feature_groups[*feature_group_index] {
				features::FeatureGroup::BagOfWords(_) => match feature {
					tangram_dataframe::Value::Number(v) => match v.partial_cmp(&0.0) {
						Some(std::cmp::Ordering::Equal) => "does not contain".to_owned(),
						_ => "contains".to_owned(),
					},
					_ => unreachable!(),
				},
				features::FeatureGroup::Normalized(_) => "".to_owned(),
				features::FeatureGroup::OneHotEncoded(_) => match feature {
					tangram_dataframe::Value::Number(v) => {
						if v.partial_cmp(&0.0) == Some(std::cmp::Ordering::Equal) {
							"false".to_owned()
						} else {
							"true".to_owned()
						}
					}
					_ => unreachable!(),
				},
				features::FeatureGroup::Identity(_) => match feature {
					tangram_dataframe::Value::Number(value) => value.to_string(),
					tangram_dataframe::Value::Enum(value) => {
						// get the name of the category
						if value.is_none() {
							"oov".to_owned()
						} else {
							let column = &dataframe.columns[*feature_group_index];
							match &column {
								tangram_dataframe::ColumnView::Enum(column) => {
									column.options[value.unwrap().get() - 1].clone()
								}
								_ => unreachable!(),
							}
						}
					}
					_ => unreachable!(),
				},
			};
			(name.replace("{}", feature.as_str()), *value)
		})
		.collect::<Vec<(String, f32)>>()
}

fn compute_feature_names(feature_groups: &[features::FeatureGroup]) -> Vec<String> {
	feature_groups
		.iter()
		.flat_map(|feature_group| match feature_group {
			features::FeatureGroup::Identity(feature_group) => {
				let name = format!("{} = '{{}}'", feature_group.source_column_name.clone());
				vec![name]
			}
			features::FeatureGroup::BagOfWords(feature_group) => {
				let name = feature_group.source_column_name.clone();
				feature_group
					.tokens
					.iter()
					.map(|(token, _)| format!("{} {{}} '{}'", name, token,))
					.collect()
			}
			features::FeatureGroup::OneHotEncoded(feature_group) => {
				let name = feature_group.source_column_name.clone();
				let mut names: Vec<String> = Vec::with_capacity(feature_group.categories.len() + 1);
				names.push(format!("{} = unknown = '{{}}'", name));
				for category in feature_group.categories.iter() {
					names.push(format!("{} = {} = '{{}}'", name, category));
				}
				names
			}
			features::FeatureGroup::Normalized(feature_group) => {
				let name = feature_group.source_column_name.clone();
				vec![name]
			}
		})
		.collect::<Vec<String>>()
}

fn compute_feature_group_map(feature_groups: &[features::FeatureGroup]) -> Vec<usize> {
	feature_groups
		.iter()
		.enumerate()
		.flat_map(|(feature_group_index, feature_group)| {
			(0..feature_group.n_features())
				.map(|_| feature_group_index)
				.collect::<Vec<usize>>()
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
					.map(column_from_column_stats)
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
					.map(column_from_column_stats)
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
									losses: Some(model.losses),
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
									losses: Some(model.losses),
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

impl TryInto<tangram_tree::Tree> for model::Tree {
	type Error = anyhow::Error;
	fn try_into(self) -> Result<tangram_tree::Tree> {
		Ok(tangram_tree::Tree {
			nodes: self
				.nodes
				.into_iter()
				.map(TryInto::try_into)
				.collect::<Result<Vec<_>>>()?,
		})
	}
}

impl TryInto<tangram_tree::Node> for model::Node {
	type Error = anyhow::Error;
	fn try_into(self) -> Result<tangram_tree::Node> {
		match self {
			Self::Branch(n) => Ok(tangram_tree::Node::Branch(tangram_tree::BranchNode {
				left_child_index: n.left_child_index.to_usize().unwrap(),
				right_child_index: n.right_child_index.to_usize().unwrap(),
				split: n.split.try_into()?,
				examples_fraction: n.examples_fraction,
			})),
			Self::Leaf(n) => Ok(tangram_tree::Node::Leaf(tangram_tree::LeafNode {
				value: n.value,
				examples_fraction: n.examples_fraction,
			})),
		}
	}
}

impl TryInto<tangram_tree::BranchSplit> for model::BranchSplit {
	type Error = anyhow::Error;
	fn try_into(self) -> Result<tangram_tree::BranchSplit> {
		match self {
			Self::Continuous(s) => Ok(tangram_tree::BranchSplit::Continuous(
				tangram_tree::BranchSplitContinuous {
					feature_index: s.feature_index.to_usize().unwrap(),
					split_value: s.split_value,
					invalid_values_direction: if s.invalid_values_direction {
						tangram_tree::SplitDirection::Right
					} else {
						tangram_tree::SplitDirection::Left
					},
				},
			)),
			Self::Discrete(s) => Ok(tangram_tree::BranchSplit::Discrete(
				tangram_tree::BranchSplitDiscrete {
					feature_index: s.feature_index.to_usize().unwrap(),
					directions: s
						.directions
						.into_iter()
						.map(TryInto::try_into)
						.collect::<Result<_, _>>()?,
				},
			)),
		}
	}
}

impl TryInto<tangram_tree::SplitDirection> for model::SplitDirection {
	type Error = anyhow::Error;
	fn try_into(self) -> Result<tangram_tree::SplitDirection> {
		Ok(match self {
			Self::Left => tangram_tree::SplitDirection::Left,
			Self::Right => tangram_tree::SplitDirection::Right,
		})
	}
}

fn column_from_column_stats(value: model::ColumnStats) -> Result<Column> {
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
