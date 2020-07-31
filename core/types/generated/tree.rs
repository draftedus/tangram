#![allow(clippy::all)]

use buffy::prelude::*;

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Tree {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub nodes: buffy::Field<Vec<Node>>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, PartialEq)]
pub enum Node {
	#[buffy(unknown)]
	UnknownVariant(u64, buffy::WireType, Vec<u8>),
	#[buffy(id = 1)]
	Branch(BranchNode),
	#[buffy(id = 2)]
	Leaf(LeafNode),
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct BranchNode {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub left_child_index: buffy::Field<u64>,
	#[buffy(id = 2)]
	pub right_child_index: buffy::Field<u64>,
	#[buffy(id = 3)]
	pub split: buffy::Field<BranchSplit>,
	/// this is the fraction of examples
	/// that passed through this branch in training
	#[buffy(id = 4)]
	pub examples_fraction: buffy::Field<f32>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, PartialEq)]
pub enum BranchSplit {
	#[buffy(unknown)]
	UnknownVariant(u64, buffy::WireType, Vec<u8>),
	#[buffy(id = 1)]
	Continuous(BranchSplitContinuous),
	#[buffy(id = 2)]
	Discrete(BranchSplitDiscrete),
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct BranchSplitContinuous {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub feature_index: buffy::Field<u64>,
	#[buffy(id = 2)]
	pub split_value: buffy::Field<f32>,
	#[buffy(id = 3)]
	pub invalid_values_direction: buffy::Field<bool>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct BranchSplitDiscrete {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub feature_index: buffy::Field<u64>,
	/// the directions correspond to label id's
	/// so the first direction is for invalid values
	#[buffy(id = 2)]
	pub directions: buffy::Field<Vec<bool>>,
}

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LeafNode {
	pub cached_size: buffy::CachedSize,
	pub unknown_fields: Vec<(u64, buffy::WireType, Vec<u8>)>,
	#[buffy(id = 1)]
	pub value: buffy::Field<f32>,
	/// this is the fraction of examples
	/// that ended in this leaf in training
	#[buffy(id = 2)]
	pub examples_fraction: buffy::Field<f32>,
}
