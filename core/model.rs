use anyhow::{format_err, Result};
use std::{
	io::{Read, Write},
	path::Path,
};
use tangram_util::id::Id;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Model {
	Regressor(Regressor),
	Classifier(Classifier),
}

impl Model {
	/// Deserialize a `Model` from a slice.
	pub fn from_slice(slice: &[u8]) -> Result<Self> {
		let major_version = slice[0];
		if major_version != 0 {
			return Err(format_err!("unknown major version {}", major_version));
		}
		let slice = &slice[1..];
		let model: Self = rmp_serde::from_slice(slice)?;
		Ok(model)
	}

	/// Deserialize a `Model` by reading the file at `path`.
	pub fn from_path(path: &Path) -> Result<Self> {
		let file = std::fs::File::open(path)?;
		let mut reader = std::io::BufReader::new(file);
		let mut major_version = [0u8; 1];
		reader.read_exact(&mut major_version)?;
		let major_version = major_version[0];
		if major_version != 0 {
			return Err(format_err!("unknown major version {}", major_version));
		}
		let model: Model = rmp_serde::from_read(&mut reader)?;
		Ok(model)
	}

	/// Write this model to the file at `path`.
	pub fn to_file(&self, path: &Path) -> Result<()> {
		let file = std::fs::File::create(path)?;
		let mut writer = std::io::BufWriter::new(file);
		writer.write_all(&[0])?;
		rmp_serde::encode::write_named(&mut writer, self)?;
		Ok(())
	}

	/// Retrieve this `Model`'s `Id`.
	pub fn id(&self) -> Id {
		match self {
			Self::Regressor(s) => s.id.parse().unwrap(),
			Self::Classifier(s) => s.id.parse().unwrap(),
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct RegressionMetrics {
	pub mse: f32,
	pub rmse: f32,
	pub mae: f32,
	pub r2: f32,
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum RegressionModel {
	Linear(LinearRegressor),
	Tree(TreeRegressor),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LinearRegressor {
	pub feature_groups: Vec<FeatureGroup>,
	pub bias: f32,
	pub weights: Vec<f32>,
	pub losses: Option<Vec<f32>>,
	pub means: Vec<f32>,
	pub train_options: LinearModelTrainOptions,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeRegressor {
	pub feature_groups: Vec<FeatureGroup>,
	pub bias: f32,
	pub trees: Vec<Tree>,
	pub losses: Vec<f32>,
	pub feature_importances: Vec<f32>,
	pub train_options: TreeModelTrainOptions,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LinearModelTrainOptions {
	pub compute_loss: bool,
	pub l2_regularization: f32,
	pub learning_rate: f32,
	pub max_epochs: u64,
	pub n_examples_per_batch: u64,
	pub early_stopping_options: Option<EarlyStoppingOptions>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeModelTrainOptions {
	pub compute_loss: bool,
	pub early_stopping_options: Option<EarlyStoppingOptions>,
	pub l2_regularization: f32,
	pub learning_rate: f32,
	pub max_depth: Option<u64>,
	pub max_examples_for_computing_bin_thresholds: usize,
	pub max_leaf_nodes: usize,
	pub max_rounds: u64,
	pub max_valid_bins_for_number_features: u8,
	pub min_examples_per_node: u64,
	pub min_gain_to_split: f32,
	pub min_sum_hessians_per_node: f32,
	pub smoothing_factor_for_discrete_bin_sorting: f32,
	pub supplemental_l2_regularization_for_discrete_splits: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EarlyStoppingOptions {
	pub early_stopping_fraction: f32,
	pub n_epochs_without_improvement_to_stop: usize,
	pub min_decrease_in_loss_for_significant_change: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum RegressionComparisonMetric {
	MeanAbsoluteError,
	MeanSquaredError,
	RootMeanSquaredError,
	R2,
}

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
	pub weights: Vec<f32>,
	pub bias: f32,
	pub classes: Vec<String>,
	pub losses: Option<Vec<f32>>,
	pub metrics: BinaryClassifierMetrics,
	pub means: Vec<f32>,
	pub train_options: LinearModelTrainOptions,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LinearMulticlassClassifier {
	pub feature_groups: Vec<FeatureGroup>,
	pub n_features: u64,
	pub n_classes: u64,
	pub biases: Vec<f32>,
	pub weights: Vec<f32>,
	pub classes: Vec<String>,
	pub losses: Option<Vec<f32>>,
	pub means: Vec<f32>,
	pub train_options: LinearModelTrainOptions,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeBinaryClassifier {
	pub feature_groups: Vec<FeatureGroup>,
	pub bias: f32,
	pub trees: Vec<Tree>,
	pub classes: Vec<String>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
	pub metrics: BinaryClassifierMetrics,
	pub train_options: TreeModelTrainOptions,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeMulticlassClassifier {
	pub feature_groups: Vec<FeatureGroup>,
	pub n_classes: u64,
	pub n_rounds: u64,
	pub biases: Vec<f32>,
	pub trees: Vec<Tree>,
	pub classes: Vec<String>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
	pub train_options: TreeModelTrainOptions,
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct StatsSettings {
	pub text_histogram_max_size: u64,
	pub number_histogram_max_size: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ColumnStats {
	Unknown(UnknownColumnStats),
	Number(NumberColumnStats),
	Enum(EnumColumnStats),
	Text(TextColumnStats),
}

impl ColumnStats {
	pub fn column_name(&self) -> String {
		match &self {
			Self::Unknown(c) => c.column_name.to_owned(),
			Self::Number(c) => c.column_name.to_owned(),
			Self::Enum(c) => c.column_name.to_owned(),
			Self::Text(c) => c.column_name.to_owned(),
		}
	}

	pub fn as_number(&self) -> Option<&NumberColumnStats> {
		match self {
			Self::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<&EnumColumnStats> {
		match self {
			Self::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<&TextColumnStats> {
		match self {
			Self::Text(s) => Some(s),
			_ => None,
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UnknownColumnStats {
	pub column_name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct NumberColumnStats {
	pub column_name: String,
	pub invalid_count: u64,
	pub unique_count: u64,
	pub histogram: Option<Vec<(f32, u64)>>,
	pub min: f32,
	pub max: f32,
	pub mean: f32,
	pub variance: f32,
	pub std: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EnumColumnStats {
	pub column_name: String,
	pub invalid_count: u64,
	pub histogram: Vec<(String, u64)>,
	pub unique_count: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TextColumnStats {
	pub column_name: String,
	pub top_tokens: Vec<TokenStats>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TokenStats {
	pub token: String,
	pub count: u64,
	pub examples_count: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum FeatureGroup {
	Identity(IdentityFeatureGroup),
	Normalized(NormalizedFeatureGroup),
	OneHotEncoded(OneHotEncodedFeatureGroup),
	BagOfWords(BagOfWordsFeatureGroup),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct IdentityFeatureGroup {
	pub source_column_name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct NormalizedFeatureGroup {
	pub source_column_name: String,
	pub mean: f32,
	pub variance: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct OneHotEncodedFeatureGroup {
	pub source_column_name: String,
	pub categories: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BagOfWordsFeatureGroup {
	pub source_column_name: String,
	pub tokenizer: Tokenizer,
	pub tokens: Vec<(String, f32)>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Tokenizer {
	Alphanumeric,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Tree {
	pub nodes: Vec<Node>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Node {
	Branch(BranchNode),
	Leaf(LeafNode),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BranchNode {
	pub left_child_index: u64,
	pub right_child_index: u64,
	pub split: BranchSplit,
	pub examples_fraction: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum BranchSplit {
	Continuous(BranchSplitContinuous),
	Discrete(BranchSplitDiscrete),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BranchSplitContinuous {
	pub feature_index: u64,
	pub split_value: f32,
	pub invalid_values_direction: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BranchSplitDiscrete {
	pub feature_index: u64,
	pub directions: Vec<SplitDirection>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum SplitDirection {
	Left,
	Right,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LeafNode {
	pub value: f32,
	pub examples_fraction: f32,
}
