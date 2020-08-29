#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Tree {
	pub nodes: Vec<Node>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum Node {
	Branch(BranchNode),
	Leaf(LeafNode),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct BranchNode {
	pub left_child_index: u64,
	pub right_child_index: u64,
	pub split: BranchSplit,
	/// this is the fraction of examples
	/// that passed through this branch in training
	pub examples_fraction: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum BranchSplit {
	Continuous(BranchSplitContinuous),
	Discrete(BranchSplitDiscrete),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct BranchSplitContinuous {
	pub feature_index: u64,
	pub split_value: f32,
	pub invalid_values_direction: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct BranchSplitDiscrete {
	pub feature_index: u64,
	/// the directions correspond to label id's
	/// so the first direction is for invalid values
	pub directions: Vec<bool>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct LeafNode {
	pub value: f32,
	/// this is the fraction of examples
	/// that ended in this leaf in training
	pub examples_fraction: f32,
}
