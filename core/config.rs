/*!
This module defines the `Config` struct, which is used to configure training a model with [`train`](../train.fn.html).
*/

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

#[derive(Debug, Clone, serde::Deserialize)]
pub enum ComparisonMetric {
	#[serde(rename = "mae")]
	MAE,
	#[serde(rename = "mse")]
	MSE,
	#[serde(rename = "rmse")]
	RMSE,
	#[serde(rename = "r2")]
	R2,
	#[serde(rename = "accuracy")]
	Accuracy,
	#[serde(rename = "auc")]
	AUC,
	#[serde(rename = "f1")]
	F1,
}

impl std::fmt::Display for ComparisonMetric {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self {
			ComparisonMetric::MAE => "Mean Absolute Error",
			ComparisonMetric::MSE => "Mean Squared Error",
			ComparisonMetric::RMSE => "Root Mean Squared Error",
			ComparisonMetric::R2 => "R2",
			ComparisonMetric::Accuracy => "Accuracy",
			ComparisonMetric::AUC => "Area Under the Receiver Operating Characteristic Curve",
			ComparisonMetric::F1 => "F1",
		};
		write!(f, "{}", s)
	}
}
