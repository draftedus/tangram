use super::production_stats::{DateWindow, DateWindowInterval};
use chrono::prelude::*;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductionMetricsResponse {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub overall: ProductionMetrics,
	pub intervals: Vec<ProductionMetrics>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductionMetrics {
	pub start_date: DateTime<Utc>,
	pub end_date: DateTime<Utc>,
	pub true_values_count: u64,
	pub prediction_metrics: Option<PredictionMetrics>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum PredictionMetrics {
	Regression(RegressionPredictionMetrics),
	Classification(ClassificationPredictionMetrics),
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegressionPredictionMetrics {
	pub mse: f32,
	pub rmse: f32,
	pub mae: f32,
	pub r2: f32,
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationPredictionMetrics {
	pub class_metrics: Vec<ClassificationPredictionClassMetrics>,
	pub accuracy: f32,
	pub baseline_accuracy: f32,
	pub precision_unweighted: f32,
	pub precision_weighted: f32,
	pub recall_unweighted: f32,
	pub recall_weighted: f32,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationPredictionClassMetrics {
	pub class_name: String,
	pub true_positives: u64,
	pub false_positives: u64,
	pub true_negatives: u64,
	pub false_negatives: u64,
	pub accuracy: f32,
	pub precision: f32,
	pub recall: f32,
	pub f1_score: f32,
}
