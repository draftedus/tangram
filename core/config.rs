/*!
This module defines the `Config` struct, which is the configuration for training a model with `tangram_core::train`.
*/

use derive_more::Display;
use std::collections::BTreeMap;

#[derive(Debug, Default, serde::Deserialize)]
pub struct Config {
	pub column_types: Option<BTreeMap<String, ColumnType>>,
	pub test_fraction: Option<f32>,
	pub grid: Option<Vec<GridItem>>,
	pub shuffle: Option<Shuffle>,
	pub comparison_metric: Option<ComparisonMetric>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ColumnType {
	#[serde(rename = "unknown")]
	Unknown,
	#[serde(rename = "number")]
	Number,
	#[serde(rename = "enum")]
	Enum { options: Vec<String> },
	#[serde(rename = "text")]
	Text,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Shuffle {
	Enabled(bool),
	Options { seed: u64 },
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "model")]
pub enum GridItem {
	#[serde(rename = "linear")]
	Linear(LinearGridItem),
	#[serde(rename = "tree")]
	Tree(TreeGridItem),
}

#[derive(Debug, serde::Deserialize)]
pub struct LinearGridItem {
	pub l2_regularization: Option<f32>,
	pub learning_rate: Option<f32>,
	pub max_epochs: Option<u64>,
	pub n_examples_per_batch: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TreeGridItem {
	pub l2_regularization: Option<f32>,
	pub learning_rate: Option<f32>,
	pub max_depth: Option<u64>,
	pub max_rounds: Option<u64>,
	pub min_examples_per_leaf: Option<u64>,
}

#[derive(Debug, Display, Clone, serde::Deserialize)]
pub enum ComparisonMetric {
	#[display(fmt = "Mean Absolute Error")]
	#[serde(rename = "mae")]
	MAE,
	#[display(fmt = "Mean Squared Error")]
	#[serde(rename = "mse")]
	MSE,
	#[display(fmt = "Root Mean Squared Error")]
	#[serde(rename = "rmse")]
	RMSE,
	#[display(fmt = "R2")]
	#[serde(rename = "r2")]
	R2,
	#[display(fmt = "Accuracy")]
	#[serde(rename = "accuracy")]
	Accuracy,
	#[display(fmt = "Area Under the Receiver Operating Characteristic Curve")]
	#[serde(rename = "auc")]
	AUC,
	#[display(fmt = "F1")]
	#[serde(rename = "f1")]
	F1,
}
