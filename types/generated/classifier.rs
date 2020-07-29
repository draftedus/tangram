#![allow(clippy::all)]

use buffy::prelude::*;

use super::features::FeatureGroup;
use super::stats::{ColumnStats, StatsSettings};
use super::train_options::{GbtModelTrainOptions, LinearModelTrainOptions};
use super::tree::Tree;

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Classifier {
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
	pub test_metrics: buffy::Field<ClassificationMetrics>,
	#[buffy(id = 13)]
	pub model: buffy::Field<ClassificationModel>,
	#[buffy(id = 14)]
	pub comparison_fraction: buffy::Field<f32>,
	#[buffy(id = 15)]
	pub comparison_metric: buffy::Field<ClassificationComparisonMetric>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct ClassificationMetrics {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub class_metrics: buffy::Field<Vec<ClassMetrics>>,
	#[buffy(id = 2)]
	pub accuracy: buffy::Field<f32>,
	#[buffy(id = 3)]
	pub precision_unweighted: buffy::Field<f32>,
	#[buffy(id = 4)]
	pub precision_weighted: buffy::Field<f32>,
	#[buffy(id = 5)]
	pub recall_unweighted: buffy::Field<f32>,
	#[buffy(id = 6)]
	pub recall_weighted: buffy::Field<f32>,
	#[buffy(id = 7)]
	pub baseline_accuracy: buffy::Field<f32>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct ClassMetrics {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub true_positives: buffy::Field<u64>,
	#[buffy(id = 2)]
	pub false_positives: buffy::Field<u64>,
	#[buffy(id = 3)]
	pub true_negatives: buffy::Field<u64>,
	#[buffy(id = 4)]
	pub false_negatives: buffy::Field<u64>,
	#[buffy(id = 5)]
	pub accuracy: buffy::Field<f32>,
	#[buffy(id = 6)]
	pub precision: buffy::Field<f32>,
	#[buffy(id = 7)]
	pub recall: buffy::Field<f32>,
	#[buffy(id = 8)]
	pub f1_score: buffy::Field<f32>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, PartialEq)]
pub enum ClassificationModel {
	#[buffy(unknown)]
	UnknownVariant(u64, buffy::WireType, Vec<u8>),
	#[buffy(id = 1)]
	LinearBinary(LinearBinaryClassifier),
	#[buffy(id = 2)]
	LinearMulticlass(LinearMulticlassClassifier),
	#[buffy(id = 3)]
	GbtBinary(GbtBinaryClassifier),
	#[buffy(id = 4)]
	GbtMulticlass(GbtMulticlassClassifier),
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LinearBinaryClassifier {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub feature_groups: buffy::Field<Vec<FeatureGroup>>,
	#[buffy(id = 2)]
	pub options: buffy::Field<LinearModelTrainOptions>,
	#[buffy(id = 3)]
	pub weights: buffy::Field<Vec<f32>>,
	#[buffy(id = 4)]
	pub bias: buffy::Field<f32>,
	#[buffy(id = 5)]
	pub classes: buffy::Field<Vec<String>>,
	#[buffy(id = 6)]
	pub losses: buffy::Field<Vec<f32>>,
	#[buffy(id = 7)]
	pub class_metrics: buffy::Field<Vec<BinaryClassifierClassMetrics>>,
	#[buffy(id = 8)]
	pub auc_roc: buffy::Field<f32>,
	#[buffy(id = 9)]
	pub means: buffy::Field<Vec<f32>>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LinearMulticlassClassifier {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub feature_groups: buffy::Field<Vec<FeatureGroup>>,
	#[buffy(id = 2)]
	pub options: buffy::Field<LinearModelTrainOptions>,
	#[buffy(id = 3)]
	pub n_features: buffy::Field<u64>,
	#[buffy(id = 4)]
	pub n_classes: buffy::Field<u64>,
	#[buffy(id = 5)]
	pub biases: buffy::Field<Vec<f32>>,
	#[buffy(id = 6)]
	pub weights: buffy::Field<Vec<f32>>,
	#[buffy(id = 7)]
	pub classes: buffy::Field<Vec<String>>,
	#[buffy(id = 8)]
	pub losses: buffy::Field<Vec<f32>>,
	#[buffy(id = 9)]
	pub means: buffy::Field<Vec<f32>>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct GbtBinaryClassifier {
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
	pub classes: buffy::Field<Vec<String>>,
	#[buffy(id = 6)]
	pub losses: buffy::Field<Vec<f32>>,
	#[buffy(id = 7)]
	pub feature_importances: buffy::Field<Vec<f32>>,
	#[buffy(id = 8)]
	pub class_metrics: buffy::Field<Vec<BinaryClassifierClassMetrics>>,
	#[buffy(id = 9)]
	pub auc_roc: buffy::Field<f32>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct GbtMulticlassClassifier {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub feature_groups: buffy::Field<Vec<FeatureGroup>>,
	#[buffy(id = 2)]
	pub options: buffy::Field<GbtModelTrainOptions>,
	#[buffy(id = 3)]
	pub n_classes: buffy::Field<u64>,
	#[buffy(id = 4)]
	pub n_rounds: buffy::Field<u64>,
	#[buffy(id = 5)]
	pub biases: buffy::Field<Vec<f32>>,
	#[buffy(id = 6)]
	pub trees: buffy::Field<Vec<Tree>>,
	#[buffy(id = 7)]
	pub classes: buffy::Field<Vec<String>>,
	#[buffy(id = 8)]
	pub losses: buffy::Field<Vec<f32>>,
	#[buffy(id = 9)]
	pub feature_importances: buffy::Field<Vec<f32>>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct BinaryClassifierClassMetrics {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub thresholds: buffy::Field<Vec<ThresholdMetrics>>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct ThresholdMetrics {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub threshold: buffy::Field<f32>,
	#[buffy(id = 2)]
	pub true_positives: buffy::Field<u64>,
	#[buffy(id = 3)]
	pub false_positives: buffy::Field<u64>,
	#[buffy(id = 4)]
	pub true_negatives: buffy::Field<u64>,
	#[buffy(id = 5)]
	pub false_negatives: buffy::Field<u64>,
	#[buffy(id = 6)]
	pub accuracy: buffy::Field<f32>,
	#[buffy(id = 7)]
	pub precision: buffy::Field<f32>,
	#[buffy(id = 8)]
	pub recall: buffy::Field<f32>,
	#[buffy(id = 9)]
	pub f1_score: buffy::Field<f32>,
	#[buffy(id = 10)]
	pub true_positive_rate: buffy::Field<f32>,
	#[buffy(id = 11)]
	pub false_positive_rate: buffy::Field<f32>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, PartialEq)]
pub enum ClassificationComparisonMetric {
	#[buffy(unknown)]
	UnknownVariant(u64, buffy::WireType, Vec<u8>),
	#[buffy(id = 1)]
	Accuracy,
	#[buffy(id = 2)]
	Aucroc,
	#[buffy(id = 3)]
	F1,
}
