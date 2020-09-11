use crate::{dataframe::*, tree};
use ndarray::prelude::*;
use num_traits::ToPrimitive;

impl tree::Tree {
	pub fn predict(&self, row: &[Value]) -> f32 {
		let mut node_index = 0;
		loop {
			match &self.nodes[node_index] {
				tree::Node::Branch(tree::BranchNode {
					left_child_index,
					right_child_index,
					split:
						tree::BranchSplit::Continuous(tree::BranchSplitContinuous {
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
							tree::SplitDirection::Left => *left_child_index,
							tree::SplitDirection::Right => *right_child_index,
						}
					} else if feature_value <= *split_value {
						*left_child_index
					} else {
						*right_child_index
					};
				}
				tree::Node::Branch(tree::BranchNode {
					left_child_index,
					right_child_index,
					split:
						tree::BranchSplit::Discrete(tree::BranchSplitDiscrete {
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
				tree::Node::Leaf(tree::LeafNode { value, .. }) => return *value,
			}
		}
	}
}

impl tree::single::TrainTree {
	pub fn predict(&self, features: ArrayView1<u8>) -> f32 {
		let mut node_index = 0;
		loop {
			match &self.nodes[node_index] {
				tree::single::TrainNode::Branch(tree::single::TrainBranchNode {
					left_child_index,
					right_child_index,
					split,
					..
				}) => match split {
					tree::single::TrainBranchSplit::Continuous(
						tree::single::TrainBranchSplitContinuous {
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
					tree::single::TrainBranchSplit::Discrete(
						tree::single::TrainBranchSplitDiscrete {
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
				tree::single::TrainNode::Leaf(tree::single::TrainLeafNode { value, .. }) => {
					return *value
				}
			}
		}
	}
}
