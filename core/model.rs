use anyhow::{anyhow, Result};
use std::{
	io::{Read, Write},
	path::Path,
};
use tangram_util::id::Id;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Model {
	#[serde(rename = "regressor")]
	Regressor(Regressor),
	#[serde(rename = "binary_classifier")]
	BinaryClassifier(BinaryClassifier),
	#[serde(rename = "multiclass_classifier")]
	MulticlassClassifier(MulticlassClassifier),
}

impl Model {
	/// Deserialize a `Model` from a slice.
	pub fn from_slice(slice: &[u8]) -> Result<Model> {
		let major_version = slice[0];
		if major_version != 0 {
			return Err(anyhow!("unknown major version {}", major_version));
		}
		let slice = &slice[1..];
		let model = rmp_serde::from_slice(slice)?;
		Ok(model)
	}

	/// Deserialize a `Model` by reading the file at `path`.
	pub fn from_path(path: &Path) -> Result<Model> {
		let file = std::fs::File::open(path)?;
		let mut reader = std::io::BufReader::new(file);
		let mut major_version = [0u8; 1];
		reader.read_exact(&mut major_version)?;
		let major_version = major_version[0];
		if major_version != 0 {
			return Err(anyhow!("unknown major version {}", major_version));
		}
		let model = rmp_serde::from_read(&mut reader)?;
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
			Model::Regressor(s) => s.id.parse().unwrap(),
			Model::BinaryClassifier(s) => s.id.parse().unwrap(),
			Model::MulticlassClassifier(s) => s.id.parse().unwrap(),
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Regressor {
	pub id: String,
	pub target_column_name: String,
	pub train_row_count: u64,
	pub test_row_count: u64,
	pub stats_settings: StatsSettings,
	pub overall_column_stats: Vec<ColumnStats>,
	pub overall_target_column_stats: ColumnStats,
	pub train_column_stats: Vec<ColumnStats>,
	pub train_target_column_stats: ColumnStats,
	pub test_column_stats: Vec<ColumnStats>,
	pub test_target_column_stats: ColumnStats,
	pub test_metrics: RegressionMetrics,
	pub baseline_metrics: RegressionMetrics,
	pub model: RegressionModel,
	pub comparison_metric: RegressionComparisonMetric,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct RegressionMetrics {
	pub mse: f32,
	pub rmse: f32,
	pub mae: f32,
	pub r2: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum RegressionModel {
	#[serde(rename = "linear")]
	Linear(LinearRegressor),
	#[serde(rename = "tree")]
	Tree(TreeRegressor),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LinearRegressor {
	pub bias: f32,
	pub weights: Vec<f32>,
	pub means: Vec<f32>,
	pub train_options: LinearModelTrainOptions,
	pub feature_groups: Vec<FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeRegressor {
	pub bias: f32,
	pub trees: Vec<Tree>,
	pub train_options: TreeModelTrainOptions,
	pub feature_groups: Vec<FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LinearModelTrainOptions {
	pub compute_loss: bool,
	pub l2_regularization: f32,
	pub learning_rate: f32,
	pub max_epochs: u64,
	pub n_examples_per_batch: u64,
	pub early_stopping_options: Option<LinearEarlyStoppingOptions>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LinearEarlyStoppingOptions {
	pub early_stopping_fraction: f32,
	pub n_epochs_without_improvement_to_stop: u64,
	pub min_decrease_in_loss_for_significant_change: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeModelTrainOptions {
	pub binned_features_layout: BinnedFeaturesLayout,
	pub compute_loss: bool,
	pub early_stopping_options: Option<TreeEarlyStoppingOptions>,
	pub l2_regularization: f32,
	pub learning_rate: f32,
	pub max_depth: Option<u64>,
	pub max_examples_for_computing_bin_thresholds: u64,
	pub max_leaf_nodes: u64,
	pub max_rounds: u64,
	pub max_valid_bins_for_number_features: u8,
	pub min_examples_per_node: u64,
	pub min_gain_to_split: f32,
	pub min_sum_hessians_per_node: f32,
	pub smoothing_factor_for_discrete_bin_sorting: f32,
	pub supplemental_l2_regularization_for_discrete_splits: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum BinnedFeaturesLayout {
	RowMajor,
	ColumnMajor,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeEarlyStoppingOptions {
	pub early_stopping_fraction: f32,
	pub n_epochs_without_improvement_to_stop: u64,
	pub min_decrease_in_loss_for_significant_change: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum RegressionComparisonMetric {
	#[serde(rename = "mean_absolute_error")]
	MeanAbsoluteError,
	#[serde(rename = "mean_squared_error")]
	MeanSquaredError,
	#[serde(rename = "root_mean_squared_error")]
	RootMeanSquaredError,
	#[serde(rename = "r2")]
	R2,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BinaryClassifier {
	pub id: String,
	pub target_column_name: String,
	pub negative_class: String,
	pub positive_class: String,
	pub train_row_count: u64,
	pub test_row_count: u64,
	pub stats_settings: StatsSettings,
	pub overall_column_stats: Vec<ColumnStats>,
	pub overall_target_column_stats: ColumnStats,
	pub train_column_stats: Vec<ColumnStats>,
	pub train_target_column_stats: ColumnStats,
	pub test_column_stats: Vec<ColumnStats>,
	pub test_target_column_stats: ColumnStats,
	pub test_metrics: BinaryClassificationMetrics,
	pub baseline_metrics: BinaryClassificationMetrics,
	pub model: BinaryClassificationModel,
	pub comparison_metric: BinaryClassificationComparisonMetric,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BinaryClassificationMetrics {
	pub auc_roc: f32,
	pub thresholds: Vec<BinaryClassificationMetricsForThreshold>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BinaryClassificationMetricsForThreshold {
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
pub enum BinaryClassificationModel {
	#[serde(rename = "linear")]
	Linear(LinearBinaryClassifier),
	#[serde(rename = "tree")]
	Tree(TreeBinaryClassifier),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LinearBinaryClassifier {
	pub bias: f32,
	pub weights: Vec<f32>,
	pub means: Vec<f32>,
	pub train_options: LinearModelTrainOptions,
	pub feature_groups: Vec<FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeBinaryClassifier {
	pub bias: f32,
	pub trees: Vec<Tree>,
	pub train_options: TreeModelTrainOptions,
	pub feature_groups: Vec<FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum BinaryClassificationComparisonMetric {
	#[serde(rename = "auc_roc")]
	AUCROC,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MulticlassClassifier {
	pub id: String,
	pub target_column_name: String,
	pub classes: Vec<String>,
	pub train_row_count: u64,
	pub test_row_count: u64,
	pub stats_settings: StatsSettings,
	pub overall_column_stats: Vec<ColumnStats>,
	pub overall_target_column_stats: ColumnStats,
	pub train_column_stats: Vec<ColumnStats>,
	pub train_target_column_stats: ColumnStats,
	pub test_column_stats: Vec<ColumnStats>,
	pub test_target_column_stats: ColumnStats,
	pub test_metrics: MulticlassClassificationMetrics,
	pub baseline_metrics: MulticlassClassificationMetrics,
	pub model: MulticlassClassificationModel,
	pub comparison_metric: MulticlassClassificationComparisonMetric,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MulticlassClassificationMetrics {
	pub class_metrics: Vec<ClassMetrics>,
	pub accuracy: f32,
	pub precision_unweighted: f32,
	pub precision_weighted: f32,
	pub recall_unweighted: f32,
	pub recall_weighted: f32,
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
pub enum MulticlassClassificationModel {
	#[serde(rename = "linear")]
	Linear(LinearMulticlassClassifier),
	#[serde(rename = "tree")]
	Tree(TreeMulticlassClassifier),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LinearMulticlassClassifier {
	pub n_features: usize,
	pub n_classes: usize,
	pub biases: Vec<f32>,
	pub weights: Vec<f32>,
	pub means: Vec<f32>,
	pub train_options: LinearModelTrainOptions,
	pub feature_groups: Vec<FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeMulticlassClassifier {
	pub n_classes: usize,
	pub n_rounds: usize,
	pub biases: Vec<f32>,
	pub trees: Vec<Tree>,
	pub train_options: TreeModelTrainOptions,
	pub feature_groups: Vec<FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum MulticlassClassificationComparisonMetric {
	#[serde(rename = "accuracy")]
	Accuracy,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct StatsSettings {
	pub token_histogram_max_size: usize,
	pub number_histogram_max_size: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ColumnStats {
	#[serde(rename = "unknown")]
	Unknown(UnknownColumnStats),
	#[serde(rename = "number")]
	Number(NumberColumnStats),
	#[serde(rename = "enum")]
	Enum(EnumColumnStats),
	#[serde(rename = "text")]
	Text(TextColumnStats),
}

impl ColumnStats {
	pub fn column_name(&self) -> String {
		match &self {
			ColumnStats::Unknown(c) => c.column_name.to_owned(),
			ColumnStats::Number(c) => c.column_name.to_owned(),
			ColumnStats::Enum(c) => c.column_name.to_owned(),
			ColumnStats::Text(c) => c.column_name.to_owned(),
		}
	}

	pub fn as_number(&self) -> Option<&NumberColumnStats> {
		match self {
			ColumnStats::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<&EnumColumnStats> {
		match self {
			ColumnStats::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<&TextColumnStats> {
		match self {
			ColumnStats::Text(s) => Some(s),
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
	pub tokenizer: Tokenizer,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TokenStats {
	pub token: Token,
	pub occurrence_count: u64,
	pub examples_count: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum Token {
	#[serde(rename = "unigram")]
	Unigram(String),
	#[serde(rename = "bigram")]
	Bigram(String, String),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum FeatureGroup {
	#[serde(rename = "identity")]
	Identity(IdentityFeatureGroup),
	#[serde(rename = "normalized")]
	Normalized(NormalizedFeatureGroup),
	#[serde(rename = "one_hot_encoded")]
	OneHotEncoded(OneHotEncodedFeatureGroup),
	#[serde(rename = "bag_of_words")]
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
	pub options: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BagOfWordsFeatureGroup {
	pub source_column_name: String,
	pub tokenizer: Tokenizer,
	pub tokens: Vec<BagOfWordsFeatureGroupToken>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BagOfWordsFeatureGroupToken {
	pub token: Token,
	pub idf: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Tokenizer {
	#[serde(rename = "alphanumeric")]
	Alphanumeric,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Tree {
	pub nodes: Vec<Node>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Node {
	#[serde(rename = "branch")]
	Branch(BranchNode),
	#[serde(rename = "leaf")]
	Leaf(LeafNode),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BranchNode {
	pub left_child_index: usize,
	pub right_child_index: usize,
	pub split: BranchSplit,
	pub examples_fraction: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum BranchSplit {
	#[serde(rename = "continuous")]
	Continuous(BranchSplitContinuous),
	#[serde(rename = "discrete")]
	Discrete(BranchSplitDiscrete),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BranchSplitContinuous {
	pub feature_index: usize,
	pub split_value: f32,
	pub invalid_values_direction: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BranchSplitDiscrete {
	pub feature_index: usize,
	pub directions: Vec<SplitDirection>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum SplitDirection {
	#[serde(rename = "left")]
	Left,
	#[serde(rename = "right")]
	Right,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LeafNode {
	pub value: f32,
	pub examples_fraction: f32,
}

impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Token::Unigram(token) => write!(f, "{}", token),
			Token::Bigram(token_a, token_b) => write!(f, "{} {}", token_a, token_b),
		}
	}
}
