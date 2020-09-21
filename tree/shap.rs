use super::{
	BranchNode, BranchSplit, BranchSplitContinuous, BranchSplitDiscrete, Node, SplitDirection, Tree,
};
use ndarray::prelude::*;
use num_traits::ToPrimitive;

// Compute the SHAP value for a single example.
pub fn compute_shap(
	example: &[tangram_dataframe::Value],
	trees: ArrayView1<Tree>,
	bias: f32,
	shap_values: &mut [f32],
) {
	let n_features = example.len();
	for tree in trees {
		let shap_values_for_tree = tree_shap(example, tree);
		for (shap_value, tree_shap_value) in shap_values.iter_mut().zip(shap_values_for_tree) {
			*shap_value += tree_shap_value;
		}
	}
	shap_values[n_features] += bias;
	for tree in trees {
		shap_values[n_features] += compute_expectation(tree, 0);
	}
}

/// This function, and the helper functions below it, are a direct port from https://github.com/slundberg/shap.
fn tree_shap(example: &[tangram_dataframe::Value], tree: &Tree) -> Vec<f32> {
	let n_features = example.len();
	let mut phi = vec![0.0; n_features + 1];
	let max_depth = max_depth(tree, 0, 0) + 2;
	let mut unique_path = vec![PathItem::new(); max_depth * (max_depth + 1) / 2];
	tree_shap_recursive(
		phi.as_mut_slice(),
		example,
		tree,
		0,
		unique_path.as_mut_slice(),
		0,
		1.0,
		1.0,
		None,
	);
	phi
}

#[derive(Debug, Clone)]
struct PathItem {
	feature_index: Option<usize>,
	zero_fraction: f32,
	one_fraction: f32,
	pweight: f32,
}

impl PathItem {
	fn new() -> Self {
		Self {
			feature_index: None,
			zero_fraction: 0.0,
			one_fraction: 0.0,
			pweight: 0.0,
		}
	}
}

#[allow(clippy::too_many_arguments)]
fn tree_shap_recursive(
	phi: &mut [f32],
	example: &[tangram_dataframe::Value],
	tree: &Tree,
	node_index: usize,
	unique_path: &mut [PathItem],
	unique_depth: usize,
	parent_zero_fraction: f32,
	parent_one_fraction: f32,
	parent_feature_index: Option<usize>,
) {
	extend_path(
		unique_path,
		unique_depth,
		parent_zero_fraction,
		parent_one_fraction,
		parent_feature_index,
	);
	let mut unique_depth = unique_depth;
	let node = &tree.nodes[node_index];
	match node {
		Node::Leaf(n) => {
			for path_index in 1..=unique_depth {
				let weight = unwound_path_sum(unique_path, unique_depth, path_index);
				let path_item = &unique_path[path_index];
				let scale = weight * (path_item.one_fraction - path_item.zero_fraction);
				phi[path_item.feature_index.unwrap()] += scale * n.value;
			}
		}
		Node::Branch(n) => {
			let (hot_child_index, cold_child_index) = compute_hot_cold_child(n, example);
			let hot_zero_fraction =
				tree.nodes[hot_child_index].examples_fraction() / n.examples_fraction;
			let cold_zero_fraction =
				tree.nodes[cold_child_index].examples_fraction() / n.examples_fraction;
			let mut incoming_zero_fraction = 1.0;
			let mut incoming_one_fraction = 1.0;
			let current_feature_index = n.split.feature_index();
			if let Some(path_index) = (1..=unique_depth)
				.find(|i| unique_path[*i].feature_index.unwrap() == current_feature_index)
			{
				incoming_zero_fraction = unique_path[path_index].zero_fraction;
				incoming_one_fraction = unique_path[path_index].one_fraction;
				unwind_path(unique_path, unique_depth, path_index);
				unique_depth -= 1;
			};
			let feature_index = n.split.feature_index();
			let (parent_path, child_path) = unique_path.split_at_mut(unique_depth + 1);
			child_path[0..parent_path.len()].clone_from_slice(parent_path);
			tree_shap_recursive(
				phi,
				example,
				tree,
				hot_child_index,
				child_path,
				unique_depth + 1,
				hot_zero_fraction * incoming_zero_fraction,
				incoming_one_fraction,
				Some(feature_index),
			);
			child_path[0..parent_path.len()].clone_from_slice(parent_path);
			tree_shap_recursive(
				phi,
				example,
				tree,
				cold_child_index,
				child_path,
				unique_depth + 1,
				cold_zero_fraction * incoming_zero_fraction,
				0.0,
				Some(feature_index),
			);
		}
	};
}

fn extend_path(
	unique_path: &mut [PathItem],
	unique_depth: usize,
	zero_fraction: f32,
	one_fraction: f32,
	feature_index: Option<usize>,
) {
	unique_path[unique_depth] = PathItem {
		feature_index,
		zero_fraction,
		one_fraction,
		pweight: if unique_depth == 0 { 1.0 } else { 0.0 },
	};
	if unique_depth == 0 {
		return;
	}
	for i in (0..unique_depth).rev() {
		unique_path[i + 1].pweight +=
			one_fraction * unique_path[i].pweight * (i + 1).to_f32().unwrap()
				/ (unique_depth + 1).to_f32().unwrap();
		unique_path[i].pweight =
			zero_fraction * unique_path[i].pweight * (unique_depth - i).to_f32().unwrap()
				/ (unique_depth + 1).to_f32().unwrap();
	}
}

fn unwind_path(unique_path: &mut [PathItem], unique_depth: usize, path_index: usize) {
	let one_fraction = unique_path[path_index].one_fraction;
	let zero_fraction = unique_path[path_index].zero_fraction;
	let mut next_one_portion = unique_path[unique_depth].pweight;
	for i in (0..unique_depth).rev() {
		if one_fraction != 0.0 {
			let tmp = unique_path[i].pweight;
			unique_path[i].pweight = next_one_portion * (unique_depth + 1).to_f32().unwrap()
				/ ((i + 1).to_f32().unwrap() * one_fraction);
			next_one_portion = tmp
				- unique_path[i].pweight * zero_fraction * (unique_depth - i).to_f32().unwrap()
					/ (unique_depth + 1).to_f32().unwrap();
		} else {
			unique_path[i].pweight = unique_path[i].pweight * (unique_depth + 1).to_f32().unwrap()
				/ (zero_fraction * (unique_depth - i).to_f32().unwrap());
		}
	}
	for i in path_index..unique_depth {
		unique_path[i].feature_index = unique_path[i + 1].feature_index;
		unique_path[i].zero_fraction = unique_path[i + 1].zero_fraction;
		unique_path[i].one_fraction = unique_path[i + 1].one_fraction;
	}
}

fn unwound_path_sum(unique_path: &[PathItem], unique_depth: usize, path_index: usize) -> f32 {
	let one_fraction = unique_path[path_index].one_fraction;
	let zero_fraction = unique_path[path_index].zero_fraction;
	let mut next_one_portion = unique_path[unique_depth].pweight;
	let mut total = 0.0;
	if one_fraction != 0.0 {
		for i in (0..unique_depth).rev() {
			let tmp = next_one_portion / ((i + 1).to_f32().unwrap() * one_fraction);
			total += tmp;
			next_one_portion =
				unique_path[i].pweight - tmp * zero_fraction * (unique_depth - i).to_f32().unwrap();
		}
	} else {
		for i in (0..unique_depth).rev() {
			total +=
				unique_path[i].pweight / (zero_fraction * (unique_depth - i).to_f32().unwrap());
		}
	}
	total * (unique_depth + 1).to_f32().unwrap()
}

fn compute_hot_cold_child(
	node: &BranchNode,
	example: &[tangram_dataframe::Value],
) -> (usize, usize) {
	match &node.split {
		BranchSplit::Continuous(BranchSplitContinuous {
			feature_index,
			split_value,
			invalid_values_direction,
		}) => match example[*feature_index] {
			tangram_dataframe::Value::Number(v) => {
				if v.is_nan() {
					if let SplitDirection::Left = invalid_values_direction {
						(node.left_child_index, node.right_child_index)
					} else {
						(node.right_child_index, node.left_child_index)
					}
				} else if v <= *split_value {
					(node.left_child_index, node.right_child_index)
				} else {
					(node.right_child_index, node.left_child_index)
				}
			}
			_ => unreachable!(),
		},
		BranchSplit::Discrete(BranchSplitDiscrete {
			feature_index,
			directions,
		}) => match example[*feature_index] {
			tangram_dataframe::Value::Enum(value) => {
				if *directions.get(value.unwrap().get()).unwrap() == SplitDirection::Left {
					(node.left_child_index, node.right_child_index)
				} else {
					(node.right_child_index, node.left_child_index)
				}
			}
			_ => unreachable!(),
		},
	}
}

fn max_depth(tree: &Tree, node_index: usize, depth: usize) -> usize {
	let current_node = &tree.nodes[node_index];
	if let Node::Leaf(_) = current_node {
		return depth;
	}
	let current_node = match current_node {
		Node::Branch(n) => n,
		_ => unreachable!(),
	};
	let left_child_index = current_node.left_child_index;
	let right_child_index = current_node.right_child_index;
	let left_depth = max_depth(tree, left_child_index, depth + 1);
	let right_depth = max_depth(tree, right_child_index, depth + 1);
	left_depth.max(right_depth) + 1
}

fn compute_expectation(tree: &Tree, node_index: usize) -> f32 {
	let current_node = &tree.nodes[node_index];
	if let Node::Leaf(n) = current_node {
		return n.value;
	}
	let current_node = match current_node {
		Node::Branch(n) => n,
		_ => unreachable!(),
	};
	let left_child_index = current_node.left_child_index;
	let right_child_index = current_node.right_child_index;
	let left_child = &tree.nodes[left_child_index];
	let right_child = &tree.nodes[right_child_index];
	let left_value = compute_expectation(tree, left_child_index);
	let right_value = compute_expectation(tree, right_child_index);
	(left_child.examples_fraction() / current_node.examples_fraction) * left_value
		+ (right_child.examples_fraction() / current_node.examples_fraction) * right_value
}
