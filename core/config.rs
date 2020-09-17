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
	pub l2_regularization: f32,
	pub learning_rate: f32,
	pub max_epochs: u64,
	pub n_examples_per_batch: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct TreeGridItem {
	pub l2_regularization: f32,
	pub learning_rate: f32,
	pub max_depth: u64,
	pub max_rounds: u64,
	pub min_examples_per_leaf: u64,
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
