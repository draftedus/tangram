use super::features::FeatureGroup;
use super::stats::{ColumnStats, StatsSettings};
use super::train_options::{LinearModelTrainOptions, TreeModelTrainOptions};
use super::tree::Tree;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Regressor {
	pub id: String,
	pub target_column_name: String,
	pub row_count: u64,
	pub stats_settings: StatsSettings,
	pub overall_column_stats: Vec<ColumnStats>,
	pub overall_target_column_stats: ColumnStats,
	pub train_column_stats: Vec<ColumnStats>,
	pub train_target_column_stats: ColumnStats,
	pub test_column_stats: Vec<ColumnStats>,
	pub test_target_column_stats: ColumnStats,
	pub test_fraction: f32,
	pub test_metrics: RegressionMetrics,
	pub model: RegressionModel,
	pub comparison_fraction: f32,
	pub comparison_metric: RegressionComparisonMetric,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct RegressionMetrics {
	pub mse: f32,
	pub rmse: f32,
	pub mae: f32,
	pub r2: f32,
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum RegressionModel {
	Linear(LinearRegressor),
	Tree(TreeRegressor),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct LinearRegressor {
	pub feature_groups: Vec<FeatureGroup>,
	pub options: LinearModelTrainOptions,
	pub bias: f32,
	pub weights: Vec<f32>,
	pub losses: Vec<f32>,
	pub means: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct TreeRegressor {
	pub feature_groups: Vec<FeatureGroup>,
	pub options: TreeModelTrainOptions,
	pub bias: f32,
	pub trees: Vec<Tree>,
	pub losses: Vec<f32>,
	pub feature_importances: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum RegressionComparisonMetric {
	MeanAbsoluteError,
	MeanSquaredError,
	RootMeanSquaredError,
	R2,
}
