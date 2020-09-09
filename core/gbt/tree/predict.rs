use crate::{dataframe::*, gbt};
use ndarray::prelude::*;
use num_traits::ToPrimitive;

impl gbt::types::Tree {
	pub fn predict(&self, row: &[Value]) -> f32 {
		let mut node_index = 0;
		loop {
			match &self.nodes[node_index] {
				gbt::types::Node::Branch(gbt::types::BranchNode {
					left_child_index,
					right_child_index,
					split:
						gbt::types::BranchSplit::Continuous(gbt::types::BranchSplitContinuous {
							feature_index,
							split_value,
							invalid_values_direction,
							..
						}),
					..
				}) => {
					let feature_value = match row[*feature_index] {
						Value::Number(value) => value,
						_ => unreachable!(),
					};
					node_index = if feature_value.is_nan() {
						match invalid_values_direction {
							gbt::types::SplitDirection::Left => *left_child_index,
							gbt::types::SplitDirection::Right => *right_child_index,
						}
					} else if feature_value <= *split_value {
						*left_child_index
					} else {
						*right_child_index
					};
				}
				gbt::types::Node::Branch(gbt::types::BranchNode {
					left_child_index,
					right_child_index,
					split:
						gbt::types::BranchSplit::Discrete(gbt::types::BranchSplitDiscrete {
							feature_index,
							directions,
							..
						}),
					..
				}) => {
					let feature_value = match row[*feature_index] {
						Value::Enum(value) => value.to_u8().unwrap(),
						_ => unreachable!(),
					};
					node_index = if !directions.get(feature_value).unwrap() {
						*left_child_index
					} else {
						*right_child_index
					};
				}
				gbt::types::Node::Leaf(gbt::types::LeafNode { value, .. }) => return *value,
			}
		}
	}
}

impl gbt::tree::types::TrainTree {
	pub fn predict(&self, features: ArrayView1<u8>) -> f32 {
		let mut node_index = 0;
		loop {
			match &self.nodes[node_index] {
				gbt::tree::types::TrainNode::Branch(gbt::tree::types::TrainBranchNode {
					left_child_index,
					right_child_index,
					split,
					..
				}) => match split {
					gbt::tree::types::TrainBranchSplit::Continuous(
						gbt::tree::types::TrainBranchSplitContinuous {
							feature_index,
							bin_index,
							..
						},
					) => {
						node_index = if features[*feature_index] <= *bin_index {
							left_child_index.unwrap()
						} else {
							right_child_index.unwrap()
						};
					}
					gbt::tree::types::TrainBranchSplit::Discrete(
						gbt::tree::types::TrainBranchSplitDiscrete {
							feature_index,
							directions,
							..
						},
					) => {
						let bin_index = features[*feature_index];
						node_index = if !directions.get(bin_index).unwrap() {
							left_child_index.unwrap()
						} else {
							right_child_index.unwrap()
						};
					}
				},
				gbt::tree::types::TrainNode::Leaf(gbt::tree::types::TrainLeafNode {
					value,
					..
				}) => return *value,
			}
		}
	}
}
