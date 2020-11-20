use tangram_app_layouts::model_layout::ModelLayoutInfo;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub id: String,
	pub metrics: TrainedModelMetrics,
	pub hyperparameters: Vec<(String, String)>,
	pub model_layout_info: ModelLayoutInfo,
}

#[derive(Clone, serde::Serialize)]
pub struct TrainedModelMetrics {
	pub identifier: String,
	pub metric: f32,
	pub model_type: String,
	pub time: String,
}

// pub enum Hyperparameters {
// 	Linear(LinearHyperparmeters),
// 	Tree(TreeHyperparameters),
// }
// struct LinearHyperparmeters {
// 	pub compute_losses: bool,
// 	pub early_stopping_options: Option<LinearEarlyStoppingOptions>,
// 	pub l2_regularization: f32,
// 	pub learning_rate: f32,
// 	pub max_epochs: usize,
// 	pub n_examples_per_batch: usize,
// }

// pub struct LinearEarlyStoppingOptions {
// 	pub early_stopping_fraction: f32,
// 	pub n_epochs_without_improvement_to_stop: usize,
// 	pub min_decrease_in_loss_for_significant_change: f32,
// }

// pub struct TreeHyperparameters {
// 	pub binned_features_layout: BinnedFeaturesLayout,
// 	pub compute_losses: bool,
// 	pub early_stopping_options: Option<TreeEarlyStoppingOptions>,
// 	pub l2_regularization: f32,
// 	pub learning_rate: f32,
// 	pub max_depth: Option<usize>,
// 	pub max_examples_for_computing_bin_thresholds: usize,
// 	pub max_leaf_nodes: usize,
// 	pub max_valid_bins_for_number_features: u8,
// 	pub max_rounds: usize,
// 	pub min_examples_per_node: usize,
// 	pub min_gain_to_split: f32,
// 	pub min_sum_hessians_per_node: f32,
// 	pub smoothing_factor_for_discrete_bin_sorting: f32,
// 	pub supplemental_l2_regularization_for_discrete_splits: f32,
// }

// pub enum BinnedFeaturesLayout {
// 	RowMajor,
// 	ColumnMajor,
// }

// pub struct TreeEarlyStoppingOptions {
// 	pub early_stopping_fraction: f32,
// 	pub n_epochs_without_improvement_to_stop: usize,
// 	pub min_decrease_in_loss_for_significant_change: f32,
// }
