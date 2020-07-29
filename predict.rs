use crate::{dataframe, features, gbt, linear, types};
use anyhow::{format_err, Result};
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PredictOptions {
	pub threshold: f32,
}

impl Default for PredictOptions {
	fn default() -> Self {
		Self { threshold: 0.5 }
	}
}

pub type PredictInput = Vec<serde_json::Map<String, serde_json::Value>>;

#[derive(serde::Serialize, Debug)]
#[serde(untagged)]
pub enum PredictOutput {
	Regression(Vec<RegressionPredictOutput>),
	Classification(Vec<ClassificationPredictOutput>),
}

#[derive(serde::Serialize, Debug)]
pub struct RegressionPredictOutput {
	pub value: f32,
	#[serde(rename = "shapValues")]
	pub shap_values: Option<ShapValues>,
}

#[derive(serde::Serialize, Debug)]
pub struct ClassificationPredictOutput {
	#[serde(rename = "className")]
	pub class_name: String,
	pub probabilities: BTreeMap<String, f32>,
	#[serde(rename = "shapValues")]
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
	GbtRegressor(GbtRegressorPredictModel),
	LinearBinaryClassifier(LinearBinaryClassifierPredictModel),
	GbtBinaryClassifier(GbtBinaryClassifierPredictModel),
	GbtMulticlassClassifier(GbtMulticlassClassifierPredictModel),
	LinearMulticlassClassifier(LinearMulticlassClassifierPredictModel),
}

#[derive(Debug)]
pub struct LinearRegressorPredictModel {
	pub id: String,
	pub columns: Vec<Column>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: linear::Regressor,
}

#[derive(Debug)]
pub struct GbtRegressorPredictModel {
	pub id: String,
	pub columns: Vec<Column>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: gbt::Regressor,
}

#[derive(Debug)]
pub struct LinearBinaryClassifierPredictModel {
	pub id: String,
	pub columns: Vec<Column>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: linear::BinaryClassifier,
}

#[derive(Debug)]
pub struct GbtBinaryClassifierPredictModel {
	pub id: String,
	pub columns: Vec<Column>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: gbt::BinaryClassifier,
}

#[derive(Debug)]
pub struct LinearMulticlassClassifierPredictModel {
	pub id: String,
	pub columns: Vec<Column>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: linear::MulticlassClassifier,
}

#[derive(Debug)]
pub struct GbtMulticlassClassifierPredictModel {
	pub id: String,
	pub columns: Vec<Column>,
	pub feature_groups: Vec<features::FeatureGroup>,
	pub model: gbt::MulticlassClassifier,
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
		PredictModel::GbtRegressor(model) => model.columns.as_slice(),
		PredictModel::LinearBinaryClassifier(model) => model.columns.as_slice(),
		PredictModel::GbtBinaryClassifier(model) => model.columns.as_slice(),
		PredictModel::LinearMulticlassClassifier(model) => model.columns.as_slice(),
		PredictModel::GbtMulticlassClassifier(model) => model.columns.as_slice(),
	};
	let column_names = columns.iter().map(|c| c.column_name()).collect();
	let column_types = columns
		.iter()
		.map(|c| match c {
			Column::Unknown(_) => dataframe::ColumnType::Unknown,
			Column::Number(_) => dataframe::ColumnType::Number,
			Column::Enum(s) => dataframe::ColumnType::Enum {
				options: s.options.clone(),
			},
			Column::Text(_) => dataframe::ColumnType::Text,
		})
		.collect();
	let mut dataframe = dataframe::DataFrame::new(column_names, column_types);

	// fill the dataframe with the input
	for input in input {
		for column in dataframe.columns.iter_mut() {
			match column {
				dataframe::Column::Unknown(column) => column.len += 1,
				dataframe::Column::Number(column) => {
					let value = match input.get(&column.name) {
						Some(serde_json::Value::Number(value)) => value.as_f64().unwrap() as f32,
						_ => std::f32::NAN,
					};
					column.data.push(value);
				}
				dataframe::Column::Enum(column) => {
					let value = input.get(&column.name).and_then(|value| value.as_str());
					let value = value
						.and_then(|value| {
							column
								.options
								.iter()
								.position(|option| option == value)
								.map(|position| position + 1)
						})
						.unwrap_or(0);
					column.data.push(value);
				}
				dataframe::Column::Text(column) => {
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
			);
			model.predict(
				features.view(),
				predictions.view_mut(),
				Some(shap_values.view_mut()),
			);
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
		PredictModel::GbtRegressor(model) => {
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
			);
			let mut predictions = unsafe { Array1::uninitialized(n_examples) };
			model.predict(
				features.view(),
				predictions.view_mut(),
				Some(shap_values.view_mut()),
			);
			let shap_values = compute_shap_values_regression_output_gbt(
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
			);
			model.predict(
				features.view(),
				probabilities.view_mut(),
				Some(shap_values.view_mut()),
			);
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
		PredictModel::GbtBinaryClassifier(model) => {
			let n_examples = dataframe.nrows();
			let feature_groups = &model.feature_groups;
			let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
			let model = &model.model;
			let mut features = unsafe { Array2::uninitialized((dataframe.nrows(), n_features)) };
			features::compute_features_ndarray_value(
				&dataframe.view(),
				feature_groups,
				features.view_mut(),
			);
			let mut probabilities = unsafe { Array2::uninitialized((n_examples, 2)) };
			let mut shap_values = Array3::zeros((features.nrows(), 1, features.ncols() + 1));
			model.predict(
				features.view(),
				probabilities.view_mut(),
				Some(shap_values.view_mut()),
			);
			let shap_values = compute_shap_values_classification_output_gbt(
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
			);
			model.predict(
				features.view(),
				probabilities.view_mut(),
				Some(shap_values.view_mut()),
			);
			let shap_values = compute_shap_values_classification_output_linear(
				feature_groups.as_slice(),
				&model.classes.as_slice().unwrap(),
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
		PredictModel::GbtMulticlassClassifier(model) => {
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
			);
			let mut probabilities = unsafe { Array2::uninitialized((n_examples, n_classes)) };
			let mut shap_values =
				Array3::zeros((features.nrows(), n_classes, features.ncols() + 1));
			model.predict(
				features.view(),
				probabilities.view_mut(),
				Some(shap_values.view_mut()),
			);
			let shap_values = compute_shap_values_classification_output_gbt(
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
	dataframe: dataframe::DataFrameView,
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

fn compute_shap_values_regression_output_gbt(
	feature_groups: &[features::FeatureGroup],
	shap_values: ArrayView3<f32>,
	features: ArrayView2<dataframe::Value>,
	dataframe: dataframe::DataFrameView,
) -> Vec<ShapValues> {
	let feature_names = compute_feature_names(feature_groups);
	let feature_group_map = compute_feature_group_map(feature_groups);
	shap_values
		.axis_iter(Axis(0))
		.zip(features.axis_iter(Axis(0)))
		.map(|(shap_values, features)| {
			let baseline = shap_values.get([0, shap_values.len() - 1]).unwrap();
			let mut shap_values = compute_shap_values_gbt(
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
	dataframe: dataframe::DataFrameView,
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

fn compute_shap_values_linear(
	feature_group_map: &[usize],
	feature_groups: &[features::FeatureGroup],
	feature_names: &[String],
	features: ArrayView1<f32>,
	shap_values: ArrayView1<f32>,
	_dataframe: &dataframe::DataFrameView,
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
						"does not contain".to_string()
					} else {
						"contains".to_string()
					}
				}
				features::FeatureGroup::Normalized(_) => "".to_string(),
				features::FeatureGroup::OneHotEncoded(_) => {
					if feature.partial_cmp(&0.0) == Some(std::cmp::Ordering::Equal) {
						"false".to_string()
					} else {
						"true".to_string()
					}
				}
				features::FeatureGroup::Identity(_) => feature.to_string(),
			};
			(name.replace("{}", feature_str.as_str()), *value)
		})
		.collect::<Vec<(String, f32)>>()
}

fn compute_shap_values_classification_output_gbt(
	feature_groups: &[features::FeatureGroup],
	classes: &[String],
	shap_values: ArrayView3<f32>,
	features: ArrayView2<dataframe::Value>,
	dataframe: dataframe::DataFrameView,
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
					let mut shap_values = compute_shap_values_gbt(
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

fn compute_shap_values_gbt(
	feature_group_map: &[usize],
	feature_groups: &[features::FeatureGroup],
	feature_names: &[String],
	features: ArrayView1<dataframe::Value>,
	shap_values: ArrayView1<f32>,
	dataframe: &dataframe::DataFrameView,
) -> Vec<(String, f32)> {
	feature_names
		.iter()
		.zip(shap_values.iter())
		.zip(features)
		.zip(feature_group_map.iter())
		.map(|(((name, value), feature), feature_group_index)| {
			let feature = match feature_groups[*feature_group_index] {
				features::FeatureGroup::BagOfWords(_) => match feature {
					dataframe::Value::Number(v) => match v.partial_cmp(&0.0) {
						Some(std::cmp::Ordering::Equal) => "does not contain".to_string(),
						_ => "contains".to_string(),
					},
					_ => unreachable!(),
				},
				features::FeatureGroup::Normalized(_) => "".to_string(),
				features::FeatureGroup::OneHotEncoded(_) => match feature {
					dataframe::Value::Number(v) => {
						if v.partial_cmp(&0.0) == Some(std::cmp::Ordering::Equal) {
							"false".to_string()
						} else {
							"true".to_string()
						}
					}
					_ => unreachable!(),
				},
				features::FeatureGroup::Identity(_) => match feature {
					dataframe::Value::Number(v) => v.to_string(),
					dataframe::Value::Enum(v) => {
						// get the name of the category
						if *v == 0 {
							"oov".to_string()
						} else {
							let column = &dataframe.columns[*feature_group_index];
							match &column {
								dataframe::ColumnView::Enum(column) => {
									column.options[*v - 1].to_string()
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
				feature_group
					.categories
					.iter()
					.for_each(|category| names.push(format!("{} = {} = '{{}}'", name, category)));
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

impl TryFrom<types::Model> for PredictModel {
	type Error = anyhow::Error;
	fn try_from(value: types::Model) -> Result<Self> {
		match value {
			types::Model::UnknownVariant(_, _, _) => Err(format_err!("unknown variant")),
			types::Model::Regressor(model) => {
				let id = model.id.required()?;
				let columns = model
					.overall_column_stats
					.required()?
					.into_iter()
					.map(column_from_column_stats)
					.collect::<Result<Vec<_>>>()?;
				match model.model.required()? {
					types::RegressionModel::Linear(model) => {
						let feature_groups = model
							.feature_groups
							.required()?
							.into_iter()
							.map(TryFrom::try_from)
							.collect::<Result<Vec<_>>>()?;
						Ok(Self::LinearRegressor(LinearRegressorPredictModel {
							id,
							columns,
							feature_groups,
							model: linear::Regressor {
								bias: model.bias.required()?,
								weights: model.weights.required()?.into(),
								means: model.means.required()?.into(),
								losses: model.losses.required()?.into(),
							},
						}))
					}
					types::RegressionModel::Gbt(model) => {
						let feature_groups = model
							.feature_groups
							.required()?
							.into_iter()
							.map(TryFrom::try_from)
							.collect::<Result<Vec<_>>>()?;
						Ok(Self::GbtRegressor(GbtRegressorPredictModel {
							id,
							columns,
							feature_groups,
							model: gbt::Regressor {
								bias: model.bias.required()?,
								trees: model
									.trees
									.required()?
									.into_iter()
									.map(TryInto::try_into)
									.collect::<Result<Vec<_>>>()?,
								feature_importances: Some(
									model.feature_importances.required()?.into(),
								),
								losses: Some(model.losses.required()?.into()),
							},
						}))
					}
					_ => unimplemented!(),
				}
			}
			types::Model::Classifier(model) => {
				let id = model.id.required()?;
				let columns = model
					.overall_column_stats
					.required()?
					.into_iter()
					.map(column_from_column_stats)
					.collect::<Result<Vec<_>>>()?;
				match model.model.required()? {
					types::ClassificationModel::UnknownVariant(_, _, _) => {
						Err(format_err!("unknown variant"))
					}
					types::ClassificationModel::LinearBinary(model) => {
						let feature_groups = model
							.feature_groups
							.required()?
							.into_iter()
							.map(TryFrom::try_from)
							.collect::<Result<Vec<_>>>()?;
						Ok(Self::LinearBinaryClassifier(
							LinearBinaryClassifierPredictModel {
								id,
								columns,
								feature_groups,
								model: linear::BinaryClassifier {
									weights: model.weights.required()?.into(),
									bias: model.bias.required()?,
									means: model.means.required()?.into(),
									losses: model.losses.required()?.into(),
									classes: model.classes.required()?.into(),
								},
							},
						))
					}
					types::ClassificationModel::GbtBinary(model) => {
						let feature_groups = model
							.feature_groups
							.required()?
							.into_iter()
							.map(TryFrom::try_from)
							.collect::<Result<Vec<_>>>()?;
						Ok(Self::GbtBinaryClassifier(GbtBinaryClassifierPredictModel {
							id,
							columns,
							feature_groups,
							model: gbt::BinaryClassifier {
								bias: model.bias.required()?,
								trees: model
									.trees
									.required()?
									.into_iter()
									.map(TryInto::try_into)
									.collect::<Result<Vec<_>>>()?,
								feature_importances: Some(
									model.feature_importances.required()?.into(),
								),
								losses: Some(model.losses.required()?.into()),
								classes: model.classes.required()?,
							},
						}))
					}
					types::ClassificationModel::LinearMulticlass(model) => {
						let n_classes = model.n_classes.required()?.to_usize().unwrap();
						let n_features = model.n_features.required()?.to_usize().unwrap();
						let weights = Array2::from_shape_vec(
							(n_features, n_classes),
							model.weights.required()?,
						)
						.unwrap();
						let feature_groups = model
							.feature_groups
							.required()?
							.into_iter()
							.map(TryFrom::try_from)
							.collect::<Result<Vec<_>>>()?;
						Ok(Self::LinearMulticlassClassifier(
							LinearMulticlassClassifierPredictModel {
								id,
								columns,
								feature_groups,
								model: linear::MulticlassClassifier {
									weights,
									biases: model.biases.required()?.into(),
									means: model.means.required()?.into(),
									losses: model.losses.required()?.into(),
									classes: model.classes.required()?.into(),
								},
							},
						))
					}
					types::ClassificationModel::GbtMulticlass(model) => {
						let feature_groups = model
							.feature_groups
							.required()?
							.into_iter()
							.map(TryFrom::try_from)
							.collect::<Result<Vec<_>>>()?;
						Ok(Self::GbtMulticlassClassifier(
							GbtMulticlassClassifierPredictModel {
								id,
								columns,
								feature_groups,
								model: gbt::MulticlassClassifier {
									biases: model.biases.required()?,
									trees: model
										.trees
										.required()?
										.into_iter()
										.map(TryInto::try_into)
										.collect::<Result<Vec<_>>>()?,
									feature_importances: Some(
										model.feature_importances.required()?.into(),
									),
									losses: Some(model.losses.required()?.into()),
									classes: model.classes.required()?,
									n_classes: model.n_classes.required()?.to_usize().unwrap(),
									n_rounds: model.n_rounds.required()?.to_usize().unwrap(),
								},
							},
						))
					}
				}
			}
		}
	}
}

impl TryInto<gbt::Tree> for types::Tree {
	type Error = anyhow::Error;
	fn try_into(self) -> Result<gbt::Tree> {
		Ok(gbt::Tree {
			nodes: self
				.nodes
				.required()?
				.into_iter()
				.map(TryInto::try_into)
				.collect::<Result<Vec<_>>>()?,
		})
	}
}

impl TryInto<gbt::Node> for types::Node {
	type Error = anyhow::Error;
	fn try_into(self) -> Result<gbt::Node> {
		match self {
			Self::UnknownVariant(_, _, _) => Err(format_err!("unknown variant")),
			Self::Branch(n) => Ok(gbt::Node::Branch(gbt::BranchNode {
				left_child_index: n.left_child_index.required()?.to_usize().unwrap(),
				right_child_index: n.right_child_index.required()?.to_usize().unwrap(),
				split: n.split.required()?.try_into()?,
				examples_fraction: n.examples_fraction.required()?,
			})),
			Self::Leaf(n) => Ok(gbt::Node::Leaf(gbt::LeafNode {
				value: n.value.required()?,
				examples_fraction: n.examples_fraction.required()?,
			})),
		}
	}
}

impl TryInto<gbt::BranchSplit> for types::BranchSplit {
	type Error = anyhow::Error;
	fn try_into(self) -> Result<gbt::BranchSplit> {
		match self {
			Self::UnknownVariant(_, _, _) => Err(format_err!("unknown variant")),
			Self::Continuous(s) => Ok(gbt::BranchSplit::Continuous(gbt::BranchSplitContinuous {
				feature_index: s.feature_index.required()?.to_usize().unwrap(),
				split_value: s.split_value.required()?,
				invalid_values_direction: if s.invalid_values_direction.required()? {
					gbt::SplitDirection::Right
				} else {
					gbt::SplitDirection::Left
				},
			})),
			Self::Discrete(s) => {
				let value_directions = s.directions.required()?;
				let mut directions =
					crate::gbt::BinDirections::new(value_directions.len().to_u8().unwrap(), false);
				value_directions
					.iter()
					.enumerate()
					.for_each(|(i, value)| directions.set(i.to_u8().unwrap(), *value));
				Ok(gbt::BranchSplit::Discrete(gbt::BranchSplitDiscrete {
					feature_index: s.feature_index.required()?.to_usize().unwrap(),
					directions,
				}))
			}
		}
	}
}

fn column_from_column_stats(value: types::ColumnStats) -> Result<Column> {
	match value {
		types::ColumnStats::UnknownVariant(_, _, _) => Err(format_err!("unknown variant")),
		types::ColumnStats::Unknown(value) => Ok(Column::Unknown(UnknownColumn {
			name: value.column_name.required()?,
		})),
		types::ColumnStats::Number(value) => Ok(Column::Number(NumberColumn {
			name: value.column_name.required()?,
		})),
		types::ColumnStats::Enum(value) => Ok(Column::Enum(EnumColumn {
			name: value.column_name.required()?,
			options: value
				.histogram
				.required()?
				.into_iter()
				.map(|v| v.0)
				.collect(),
		})),
		types::ColumnStats::Text(value) => Ok(Column::Text(TextColumn {
			name: value.column_name.required()?,
		})),
	}
}

impl TryFrom<types::FeatureGroup> for features::FeatureGroup {
	type Error = anyhow::Error;
	fn try_from(value: types::FeatureGroup) -> Result<Self> {
		match value {
			types::FeatureGroup::UnknownVariant(_, _, _) => Err(format_err!("unknown variant")),
			types::FeatureGroup::Identity(f) => Ok(features::FeatureGroup::Identity(
				features::IdentityFeatureGroup {
					source_column_name: f.source_column_name.required()?,
				},
			)),
			types::FeatureGroup::Normalized(f) => Ok(features::FeatureGroup::Normalized(
				features::NormalizedFeatureGroup {
					source_column_name: f.source_column_name.required()?,
					mean: f.mean.required()?,
					variance: f.variance.required()?,
				},
			)),
			types::FeatureGroup::OneHotEncoded(f) => Ok(features::FeatureGroup::OneHotEncoded(
				features::OneHotEncodedFeatureGroup {
					source_column_name: f.source_column_name.required()?,
					categories: f.categories.required()?,
				},
			)),
			types::FeatureGroup::BagOfWords(f) => Ok(features::FeatureGroup::BagOfWords(
				features::BagOfWordsFeatureGroup {
					source_column_name: f.source_column_name.required()?,
					tokenizer: f.tokenizer.required()?.try_into()?,
					tokens: f.tokens.required()?,
				},
			)),
		}
	}
}

impl TryFrom<types::Tokenizer> for features::Tokenizer {
	type Error = anyhow::Error;
	fn try_from(value: types::Tokenizer) -> Result<features::Tokenizer> {
		match value {
			types::Tokenizer::UnknownVariant(_, _, _) => Err(format_err!("unknown variant")),
			types::Tokenizer::Alphanumeric => Ok(features::Tokenizer::Alphanumeric),
		}
	}
}
