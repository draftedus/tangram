#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LinearModelTrainOptions {
	pub max_epochs: u64,
	pub n_examples_per_batch: u64,
	pub learning_rate: f32,
	pub l2_regularization: f32,
	pub early_stopping_fraction: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeModelTrainOptions {
	pub depth: u64,
	pub learning_rate: f32,
	pub min_examples_per_leaf: u64,
	pub max_rounds: u64,
	pub early_stopping_fraction: f32,
}
