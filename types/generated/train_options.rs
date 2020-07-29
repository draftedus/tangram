#![allow(clippy::all)]

use buffy::prelude::*;

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LinearModelTrainOptions {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub max_epochs: buffy::Field<u64>,
	#[buffy(id = 2)]
	pub n_examples_per_batch: buffy::Field<u64>,
	#[buffy(id = 3)]
	pub learning_rate: buffy::Field<f32>,
	#[buffy(id = 4)]
	pub l2_regularization: buffy::Field<f32>,
	#[buffy(id = 5)]
	pub early_stopping_fraction: buffy::Field<f32>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct GbtModelTrainOptions {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub depth: buffy::Field<u64>,
	#[buffy(id = 2)]
	pub learning_rate: buffy::Field<f32>,
	#[buffy(id = 3)]
	pub min_examples_per_leaf: buffy::Field<u64>,
	#[buffy(id = 4)]
	pub max_rounds: buffy::Field<u64>,
	#[buffy(id = 5)]
	pub early_stopping_fraction: buffy::Field<f32>,
}
