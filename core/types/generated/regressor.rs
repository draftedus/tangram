#![allow(clippy::all)]

use buffy::prelude::*;

use super::features::FeatureGroup;
use super::stats::{ColumnStats, StatsSettings};
use super::train_options::{GbtModelTrainOptions, LinearModelTrainOptions};
use super::tree::Tree;

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Regressor {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub id: buffy::Field<String>,
	#[buffy(id = 2)]
	pub target_column_name: buffy::Field<String>,
	#[buffy(id = 3)]
	pub row_count: buffy::Field<u64>,
	#[buffy(id = 4)]
	pub stats_settings: buffy::Field<StatsSettings>,
	#[buffy(id = 5)]
	pub overall_column_stats: buffy::Field<Vec<ColumnStats>>,
	#[buffy(id = 6)]
	pub overall_target_column_stats: buffy::Field<ColumnStats>,
	#[buffy(id = 7)]
	pub train_column_stats: buffy::Field<Vec<ColumnStats>>,
	#[buffy(id = 8)]
	pub train_target_column_stats: buffy::Field<ColumnStats>,
	#[buffy(id = 9)]
	pub test_column_stats: buffy::Field<Vec<ColumnStats>>,
	#[buffy(id = 10)]
	pub test_target_column_stats: buffy::Field<ColumnStats>,
	#[buffy(id = 11)]
	pub test_fraction: buffy::Field<f32>,
	#[buffy(id = 12)]
	pub test_metrics: buffy::Field<RegressionMetrics>,
	#[buffy(id = 13)]
	pub model: buffy::Field<RegressionModel>,
	#[buffy(id = 14)]
	pub comparison_fraction: buffy::Field<f32>,
	#[buffy(id = 15)]
	pub comparison_metric: buffy::Field<RegressionComparisonMetric>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct RegressionMetrics {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub mse: buffy::Field<f32>,
	#[buffy(id = 2)]
	pub rmse: buffy::Field<f32>,
	#[buffy(id = 3)]
	pub mae: buffy::Field<f32>,
	#[buffy(id = 4)]
	pub r2: buffy::Field<f32>,
	#[buffy(id = 5)]
	pub baseline_mse: buffy::Field<f32>,
	#[buffy(id = 6)]
	pub baseline_rmse: buffy::Field<f32>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, PartialEq)]
pub enum RegressionModel {
	#[buffy(unknown)]
	UnknownVariant(u64, buffy::WireType, Vec<u8>),
	#[buffy(id = 1)]
	Linear(LinearRegressor),
	#[buffy(id = 2)]
	Gbt(GbtRegressor),
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LinearRegressor {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub feature_groups: buffy::Field<Vec<FeatureGroup>>,
	#[buffy(id = 2)]
	pub options: buffy::Field<LinearModelTrainOptions>,
	#[buffy(id = 3)]
	pub bias: buffy::Field<f32>,
	#[buffy(id = 4)]
	pub weights: buffy::Field<Vec<f32>>,
	#[buffy(id = 5)]
	pub losses: buffy::Field<Vec<f32>>,
	#[buffy(id = 6)]
	pub means: buffy::Field<Vec<f32>>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct GbtRegressor {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub feature_groups: buffy::Field<Vec<FeatureGroup>>,
	#[buffy(id = 2)]
	pub options: buffy::Field<GbtModelTrainOptions>,
	#[buffy(id = 3)]
	pub bias: buffy::Field<f32>,
	#[buffy(id = 4)]
	pub trees: buffy::Field<Vec<Tree>>,
	#[buffy(id = 5)]
	pub losses: buffy::Field<Vec<f32>>,
	#[buffy(id = 6)]
	pub feature_importances: buffy::Field<Vec<f32>>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, PartialEq)]
pub enum RegressionComparisonMetric {
	#[buffy(unknown)]
	UnknownVariant(u64, buffy::WireType, Vec<u8>),
	#[buffy(id = 1)]
	MeanAbsoluteError,
	#[buffy(id = 2)]
	MeanSquaredError,
	#[buffy(id = 3)]
	RootMeanSquaredError,
	#[buffy(id = 4)]
	R2,
}
