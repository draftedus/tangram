use super::{
	features::FeatureGroup,
	stats::{ColumnStats, StatsSettings},
	train_options::{LinearModelTrainOptions, TreeModelTrainOptions},
	tree::Tree,
};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Classifier {
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
	pub test_metrics: ClassificationMetrics,
	pub model: ClassificationModel,
	pub comparison_fraction: f32,
	pub comparison_metric: ClassificationComparisonMetric,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ClassificationMetrics {
	pub class_metrics: Vec<ClassMetrics>,
	pub accuracy: f32,
	pub precision_unweighted: f32,
	pub precision_weighted: f32,
	pub recall_unweighted: f32,
	pub recall_weighted: f32,
	pub baseline_accuracy: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ClassMetrics {
	pub true_positives: u64,
	pub false_positives: u64,
	pub true_negatives: u64,
	pub false_negatives: u64,
	pub accuracy: f32,
	pub precision: f32,
	pub recall: f32,
	pub f1_score: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ClassificationModel {
	LinearBinary(LinearBinaryClassifier),
	LinearMulticlass(LinearMulticlassClassifier),
	TreeBinary(TreeBinaryClassifier),
	TreeMulticlass(TreeMulticlassClassifier),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LinearBinaryClassifier {
	pub feature_groups: Vec<FeatureGroup>,
	pub options: LinearModelTrainOptions,
	pub weights: Vec<f32>,
	pub bias: f32,
	pub classes: Vec<String>,
	pub losses: Vec<f32>,
	pub metrics: BinaryClassifierMetrics,
	pub means: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LinearMulticlassClassifier {
	pub feature_groups: Vec<FeatureGroup>,
	pub options: LinearModelTrainOptions,
	pub n_features: u64,
	pub n_classes: u64,
	pub biases: Vec<f32>,
	pub weights: Vec<f32>,
	pub classes: Vec<String>,
	pub losses: Vec<f32>,
	pub means: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeBinaryClassifier {
	pub feature_groups: Vec<FeatureGroup>,
	pub options: TreeModelTrainOptions,
	pub bias: f32,
	pub trees: Vec<Tree>,
	pub classes: Vec<String>,
	pub losses: Vec<f32>,
	pub feature_importances: Vec<f32>,
	pub metrics: BinaryClassifierMetrics,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeMulticlassClassifier {
	pub feature_groups: Vec<FeatureGroup>,
	pub options: TreeModelTrainOptions,
	pub n_classes: u64,
	pub n_rounds: u64,
	pub biases: Vec<f32>,
	pub trees: Vec<Tree>,
	pub classes: Vec<String>,
	pub losses: Vec<f32>,
	pub feature_importances: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BinaryClassifierMetrics {
	pub thresholds: Vec<ThresholdMetrics>,
	pub auc_roc: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ThresholdMetrics {
	pub threshold: f32,
	pub true_positives: u64,
	pub false_positives: u64,
	pub true_negatives: u64,
	pub false_negatives: u64,
	pub accuracy: f32,
	pub precision: f32,
	pub recall: f32,
	pub f1_score: f32,
	pub true_positive_rate: f32,
	pub false_positive_rate: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ClassificationComparisonMetric {
	Accuracy,
	Aucroc,
	F1,
}

impl Classifier {
	pub fn classes(&self) -> &[String] {
		match &self.model {
			ClassificationModel::LinearBinary(model) => model.classes.as_slice(),
			ClassificationModel::TreeBinary(model) => model.classes.as_slice(),
			ClassificationModel::LinearMulticlass(model) => model.classes.as_slice(),
			ClassificationModel::TreeMulticlass(model) => model.classes.as_slice(),
		}
	}
}
