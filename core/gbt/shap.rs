use crate::{dataframe, gbt::types};
use ndarray::prelude::*;

// computes for a single output
pub fn compute_shap(
	_example: &[dataframe::Value],
	_trees: ArrayView1<types::Tree>,
	_bias: f32,
) -> Array1<f32> {
	// let n_features = example.len();
	// let mut shap_values = trees
	// 	.axis_iter(Axis(0))
	// 	.fold(
	// 		|| vec![0.0; n_features + 1],
	// 		|mut shap_values, tree| {
	// 			let t_shap = tree_shap(example, tree.into_scalar());
	// 			shap_values
	// 				.iter_mut()
	// 				.zip(t_shap)
	// 				.for_each(|(a, b)| *a += b);
	// 			shap_values
	// 		},
	// 	)
	// 	.reduce(
	// 		|| vec![0.0; n_features + 1],
	// 		|mut a, b| {
	// 			a.iter_mut().zip(b).for_each(|(a, b)| *a += b);
	// 			a
	// 		},
	// 	);
	// shap_values[n_features] += bias;
	// trees.iter().for_each(|tree| {
	// 	shap_values[n_features] += compute_expectation(tree, 0);
	// });
	// shap_values.into()
	todo!()
}

// fn tree_shap(example: &[dataframe::Value], tree: &types::Tree) -> Vec<f32> {
// 	let n_features = example.len();
// 	let mut phi = vec![0.0; n_features + 1];
// 	let max_depth = max_depth(tree, 0, 0) + 2;
// 	let mut unique_path = vec![PathItem::new(); max_depth * (max_depth + 1) / 2];
// 	tree_shap_recursive(
// 		phi.as_mut_slice(),
// 		example,
// 		tree,
// 		0,
// 		unique_path.as_mut_slice(),
// 		0,
// 		1.0,
// 		1.0,
// 		None,
// 	);
// 	phi
// }

// #[derive(Debug, Clone)]
// struct PathItem {
// 	feature_index: Option<usize>,
// 	zero_fraction: f32,
// 	// always either 1 or 0 depending on whether this item is on the "hot" path for the example
// 	one_fraction: f32,
// 	// pweight of i'th path element is the permutation weight of the paths with i-1 ones
// 	pweight: f32,
// }

// impl PathItem {
// 	fn new() -> Self {
// 		Self {
// 			feature_index: None,
// 			zero_fraction: 0.0,
// 			one_fraction: 0.0,
// 			pweight: 0.0,
// 		}
// 	}
// }

// #[allow(unconditional_recursion, clippy::too_many_arguments)]
// fn tree_shap_recursive(
// 	phi: &mut [f32],
// 	example: &[dataframe::Value],
// 	tree: &types::Tree,
// 	node_index: usize,
// 	unique_path: &mut [PathItem],
// 	unique_depth: usize,
// 	parent_zero_fraction: f32,
// 	parent_one_fraction: f32,
// 	parent_feature_index: Option<usize>,
// ) {
// 	extend_path(
// 		unique_path,
// 		unique_depth,
// 		parent_zero_fraction,
// 		parent_one_fraction,
// 		parent_feature_index,
// 	);

// 	let mut unique_depth = unique_depth;
// 	let node = &tree.nodes[node_index];

// 	match node {
// 		types::Node::Leaf(n) => (1..=unique_depth).for_each(|path_index| {
// 			let weight = unwound_path_sum(unique_path, unique_depth, path_index);
// 			let path_item = &unique_path[path_index];
// 			let scale = weight * (path_item.one_fraction - path_item.zero_fraction);
// 			phi[path_item.feature_index.unwrap()] += scale * n.value;
// 		}),
// 		types::Node::Branch(n) => {
// 			let (hot_child_index, cold_child_index) = compute_hot_cold_child(n, example);
// 			let hot_zero_fraction =
// 				tree.nodes[hot_child_index].examples_fraction() / n.examples_fraction;
// 			let cold_zero_fraction =
// 				tree.nodes[cold_child_index].examples_fraction() / n.examples_fraction;
// 			let mut incoming_zero_fraction = 1.0;
// 			let mut incoming_one_fraction = 1.0;

// 			let current_feature_index = n.split.feature_index();
// 			if let Some(path_index) = (1..=unique_depth)
// 				.find(|i| unique_path[*i].feature_index.unwrap() == current_feature_index)
// 			{
// 				incoming_zero_fraction = unique_path[path_index].zero_fraction;
// 				incoming_one_fraction = unique_path[path_index].one_fraction;
// 				unwind_path(unique_path, unique_depth, path_index);
// 				unique_depth -= 1;
// 			};

// 			let feature_index = n.split.feature_index();
// 			let (parent_path, child_path) = unique_path.split_at_mut(unique_depth + 1);
// 			child_path[0..parent_path.len()].clone_from_slice(parent_path);
// 			tree_shap_recursive(
// 				phi,
// 				example,
// 				tree,
// 				hot_child_index,
// 				child_path,
// 				unique_depth + 1,
// 				hot_zero_fraction * incoming_zero_fraction,
// 				incoming_one_fraction,
// 				Some(feature_index),
// 			);
// 			child_path[0..parent_path.len()].clone_from_slice(parent_path);
// 			tree_shap_recursive(
// 				phi,
// 				example,
// 				tree,
// 				cold_child_index,
// 				child_path,
// 				unique_depth + 1,
// 				cold_zero_fraction * incoming_zero_fraction,
// 				0.0,
// 				Some(feature_index),
// 			);
// 		}
// 	};
// }

// fn extend_path(
// 	unique_path: &mut [PathItem],
// 	unique_depth: usize,
// 	zero_fraction: f32,
// 	one_fraction: f32,
// 	feature_index: Option<usize>,
// ) {
// 	unique_path[unique_depth] = PathItem {
// 		feature_index,
// 		zero_fraction,
// 		one_fraction,
// 		pweight: if unique_depth == 0 { 1.0 } else { 0.0 },
// 	};

// 	if unique_depth == 0 {
// 		return;
// 	}
// 	(0..unique_depth).rev().for_each(|i| {
// 		unique_path[i + 1].pweight +=
// 			one_fraction * unique_path[i].pweight * (i + 1).to_f32().unwrap()
// 				/ (unique_depth + 1).to_f32().unwrap();
// 		unique_path[i].pweight =
// 			zero_fraction * unique_path[i].pweight * (unique_depth - i).to_f32().unwrap()
// 				/ (unique_depth + 1).to_f32().unwrap();
// 	});
// }

// fn unwind_path(unique_path: &mut [PathItem], unique_depth: usize, path_index: usize) {
// 	let one_fraction = unique_path[path_index].one_fraction;
// 	let zero_fraction = unique_path[path_index].zero_fraction;
// 	let mut next_one_portion = unique_path[unique_depth].pweight;

// 	(0..unique_depth).rev().for_each(|i| {
// 		if one_fraction != 0.0 {
// 			let tmp = unique_path[i].pweight;
// 			unique_path[i].pweight = next_one_portion * (unique_depth + 1).to_f32().unwrap()
// 				/ ((i + 1).to_f32().unwrap() * one_fraction);
// 			next_one_portion = tmp
// 				- unique_path[i].pweight * zero_fraction * (unique_depth - i).to_f32().unwrap()
// 					/ (unique_depth + 1).to_f32().unwrap();
// 		} else {
// 			unique_path[i].pweight = unique_path[i].pweight * (unique_depth + 1).to_f32().unwrap()
// 				/ (zero_fraction * (unique_depth - i).to_f32().unwrap());
// 		}
// 	});

// 	(path_index..unique_depth).for_each(|i| {
// 		unique_path[i].feature_index = unique_path[i + 1].feature_index;
// 		unique_path[i].zero_fraction = unique_path[i + 1].zero_fraction;
// 		unique_path[i].one_fraction = unique_path[i + 1].one_fraction;
// 	});
// }

// fn unwound_path_sum(unique_path: &[PathItem], unique_depth: usize, path_index: usize) -> f32 {
// 	let one_fraction = unique_path[path_index].one_fraction;
// 	let zero_fraction = unique_path[path_index].zero_fraction;
// 	let mut next_one_portion = unique_path[unique_depth].pweight;
// 	let mut total = 0.0;

// 	if one_fraction != 0.0 {
// 		(0..unique_depth).rev().for_each(|i| {
// 			let tmp = next_one_portion / ((i + 1).to_f32().unwrap() * one_fraction);
// 			total += tmp;
// 			next_one_portion =
// 				unique_path[i].pweight - tmp * zero_fraction * (unique_depth - i).to_f32().unwrap();
// 		})
// 	} else {
// 		(0..unique_depth).rev().for_each(|i| {
// 			total +=
// 				unique_path[i].pweight / (zero_fraction * (unique_depth - i).to_f32().unwrap());
// 		})
// 	}

// 	total * (unique_depth + 1).to_f32().unwrap()
// }

// fn compute_hot_cold_child(
// 	node: &types::BranchNode,
// 	example: &[dataframe::Value],
// ) -> (usize, usize) {
// 	match &node.split {
// 		types::BranchSplit::Continuous(types::BranchSplitContinuous {
// 			feature_index,
// 			split_value,
// 			invalid_values_direction,
// 		}) => match example[*feature_index] {
// 			dataframe::Value::Number(v) => {
// 				if v.is_nan() {
// 					if let types::SplitDirection::Left = invalid_values_direction {
// 						(node.left_child_index, node.right_child_index)
// 					} else {
// 						(node.right_child_index, node.left_child_index)
// 					}
// 				} else if v <= *split_value {
// 					(node.left_child_index, node.right_child_index)
// 				} else {
// 					(node.right_child_index, node.left_child_index)
// 				}
// 			}
// 			_ => unreachable!(),
// 		},
// 		types::BranchSplit::Discrete(types::BranchSplitDiscrete {
// 			feature_index,
// 			directions,
// 		}) => match example[*feature_index] {
// 			dataframe::Value::Enum(v) => {
// 				if !directions.get(v.to_u8().unwrap()).unwrap() {
// 					(node.left_child_index, node.right_child_index)
// 				} else {
// 					(node.right_child_index, node.left_child_index)
// 				}
// 			}
// 			_ => unreachable!(),
// 		},
// 	}
// }

// fn max_depth(tree: &types::Tree, node_index: usize, depth: usize) -> usize {
// 	let current_node = &tree.nodes[node_index];
// 	if let types::Node::Leaf(_) = current_node {
// 		return depth;
// 	}

// 	let current_node = match current_node {
// 		types::Node::Branch(n) => n,
// 		_ => unreachable!(),
// 	};

// 	let left_child_index = current_node.left_child_index;
// 	let right_child_index = current_node.right_child_index;

// 	let left_depth = max_depth(tree, left_child_index, depth + 1);
// 	let right_depth = max_depth(tree, right_child_index, depth + 1);

// 	left_depth.max(right_depth) + 1
// }

// fn compute_expectation(tree: &types::Tree, node_index: usize) -> f32 {
// 	let current_node = &tree.nodes[node_index];
// 	if let types::Node::Leaf(n) = current_node {
// 		return n.value;
// 	}

// 	let current_node = match current_node {
// 		types::Node::Branch(n) => n,
// 		_ => unreachable!(),
// 	};

// 	let left_child_index = current_node.left_child_index;
// 	let right_child_index = current_node.right_child_index;
// 	let left_child = &tree.nodes[left_child_index];
// 	let right_child = &tree.nodes[right_child_index];

// 	let left_value = compute_expectation(tree, left_child_index);
// 	let right_value = compute_expectation(tree, right_child_index);

// 	(left_child.examples_fraction() / current_node.examples_fraction) * left_value
// 		+ (right_child.examples_fraction() / current_node.examples_fraction) * right_value
// }

#[cfg(test)]
mod brute {
	// use crate::{dataframe, gbt::types};
	// use ndarray::prelude::*;
	// use num_traits::ToPrimitive;

	// fn compute_shap_brute_regressor(
	// 	example: &[dataframe::Value],
	// 	model: &types::Regressor,
	// ) -> Array1<f32> {
	// 	let shap_values = compute_shap_brute(example, &model.trees, model.bias);
	// 	Array::from(shap_values)
	// }

	// fn compute_shap_brute_binary_classifier(
	// 	example: &[dataframe::Value],
	// 	model: &types::BinaryClassifier,
	// ) -> Array1<f32> {
	// 	let shap_values = compute_shap_brute(example, &model.trees, model.bias);
	// 	Array::from(shap_values)
	// }

	// /// returns a 2d array of shape (n_classes, n_features + 1)
	// fn compute_shap_brute_multiclass_classifier(
	// 	example: &[dataframe::Value],
	// 	model: &types::MulticlassClassifier,
	// ) -> Array2<f32> {
	// 	let n_features = example.len();
	// 	let n_classes = model.n_classes;
	// 	let mut shap_values = Array2::zeros((n_classes, n_features + 1));
	// 	let trees = ArrayView2::from_shape((n_classes, model.n_rounds), &model.trees);
	// 	(shap_values.axis_iter_mut(Axis(0)), trees, &model.biases)
	// 		.into_par_iter()
	// 		.for_each(|(mut shap, trees, bias)| {
	// 			let shap_values_for_class =
	// 				compute_shap_brute(example, trees.as_slice().unwrap(), *bias);
	// 			shap.iter_mut()
	// 				.zip(shap_values_for_class)
	// 				.for_each(|(a, b)| *a = b);
	// 		});
	// 	shap_values
	// }

	// 	// compute shap for a single class
	// 	pub fn compute_shap_brute(
	// 		example: &[dataframe::Value],
	// 		trees: ArrayView1<types::Tree>,
	// 		bias: f32,
	// 	) -> Array1<f32> {
	// 		let n_features = example.len();
	// 		let mut feature_importances = trees
	// 			.axis_iter(Axis(0))
	// 			.into_par_iter()
	// 			.map(|tree| compute_shap_brute_tree(example, tree.into_scalar()))
	// 			.reduce(
	// 				|| vec![0.0; n_features + 1],
	// 				|mut feature_importances, tree_feature_importances| {
	// 					feature_importances
	// 						.iter_mut()
	// 						.zip(tree_feature_importances.iter())
	// 						.for_each(|(a, b)| *a += b);
	// 					feature_importances
	// 				},
	// 			);
	// 		feature_importances[n_features] += bias.to_f64().unwrap();

	// 		let fi: Vec<f32> = feature_importances
	// 			.into_iter()
	// 			.map(|fi| fi.to_f32().unwrap())
	// 			.collect();

	// 		arr1(&fi)
	// 	}

	// 	/// Computes the shap values for each feature using the brute force method
	// 	fn compute_shap_brute_tree(example: &[dataframe::Value], tree: &types::Tree) -> Vec<f64> {
	// 		let n_features = example.len();
	// 		let mut feature_importances = (0..n_features)
	// 			.into_par_iter()
	// 			.map(|feature_index| {
	// 				// iterate over all subsets without feature index
	// 				let p = PowersetIter::new((0..n_features).collect());
	// 				p.filter(|s| !s.contains(&feature_index))
	// 					.map(|s| {
	// 						let mut subset = s.clone();
	// 						let subset_size = s.len();

	// 						let numerator = factorial(subset_size).to_f64().unwrap()
	// 							* (factorial(n_features - subset_size - 1).to_f64().unwrap());
	// 						let denominator = factorial(n_features).to_f64().unwrap();
	// 						let shap_weight = numerator / denominator;

	// 						let subset_value_without_feature =
	// 							compute_subset_value(example, tree, subset.as_slice(), 0);
	// 						subset.push(feature_index);
	// 						let subset_value_with_feature =
	// 							compute_subset_value(example, tree, subset.as_slice(), 0);

	// 						shap_weight * (subset_value_with_feature - subset_value_without_feature)
	// 					})
	// 					.sum()
	// 			})
	// 			.collect::<Vec<f64>>();
	// 		let empty = vec![];
	// 		let feature_importance_0 = compute_subset_value(example, tree, empty.as_slice(), 0);
	// 		feature_importances.push(feature_importance_0);
	// 		feature_importances
	// 	}

	// 	fn compute_subset_value(
	// 		example: &[dataframe::Value],
	// 		tree: &types::Tree,
	// 		subset: &[usize],
	// 		node_index: usize,
	// 	) -> f64 {
	// 		let node = &tree.nodes[node_index];
	// 		match node {
	// 			types::Node::Branch(types::BranchNode {
	// 				left_child_index,
	// 				right_child_index,
	// 				split:
	// 					types::BranchSplit::Continuous(types::BranchSplitContinuous {
	// 						feature_index,
	// 						split_value,
	// 						invalid_values_direction,
	// 						..
	// 					}),
	// 				..
	// 			}) => {
	// 				let feature_value = match example[*feature_index] {
	// 					dataframe::Value::Number(value) => value,
	// 					_ => unreachable!(),
	// 				};
	// 				// if the feature is in the subset
	// 				if subset.contains(feature_index) {
	// 					let child_node_index = if feature_value.is_nan() {
	// 						match invalid_values_direction {
	// 							types::SplitDirection::Left => *left_child_index,
	// 							types::SplitDirection::Right => *right_child_index,
	// 						}
	// 					} else if feature_value <= *split_value {
	// 						*left_child_index
	// 					} else {
	// 						*right_child_index
	// 					};
	// 					compute_subset_value(example, tree, subset, child_node_index)
	// 				} else {
	// 					let left_node_examples_fraction =
	// 						tree.nodes[*left_child_index].examples_fraction();
	// 					let right_node_examples_fraction =
	// 						tree.nodes[*right_child_index].examples_fraction();
	// 					let node_examples_fraction = node.examples_fraction();
	// 					let left_weight = left_node_examples_fraction.to_f64().unwrap()
	// 						/ node_examples_fraction.to_f64().unwrap();
	// 					let right_weight = right_node_examples_fraction.to_f64().unwrap()
	// 						/ node_examples_fraction.to_f64().unwrap();
	// 					compute_subset_value(example, tree, subset, *left_child_index) * left_weight
	// 						+ compute_subset_value(example, tree, subset, *right_child_index)
	// 							* right_weight
	// 				}
	// 			}
	// 			types::Node::Branch(types::BranchNode {
	// 				left_child_index,
	// 				right_child_index,
	// 				split:
	// 					types::BranchSplit::Discrete(types::BranchSplitDiscrete {
	// 						feature_index,
	// 						directions,
	// 						..
	// 					}),
	// 				..
	// 			}) => {
	// 				let feature_value = match example[*feature_index] {
	// 					dataframe::Value::Enum(value) => value.to_u8().unwrap(),
	// 					_ => unreachable!(),
	// 				};

	// 				// if the feature is in the subset
	// 				if subset.contains(feature_index) {
	// 					let child_node_index = if !directions.get(feature_value).unwrap() {
	// 						*left_child_index
	// 					} else {
	// 						*right_child_index
	// 					};
	// 					compute_subset_value(example, tree, subset, child_node_index)
	// 				} else {
	// 					let left_node_examples_fraction =
	// 						tree.nodes[*left_child_index].examples_fraction();
	// 					let right_node_examples_fraction =
	// 						tree.nodes[*right_child_index].examples_fraction();
	// 					let node_examples_fraction = node.examples_fraction();
	// 					let left_weight = left_node_examples_fraction.to_f64().unwrap()
	// 						/ node_examples_fraction.to_f64().unwrap();
	// 					let right_weight = right_node_examples_fraction.to_f64().unwrap()
	// 						/ node_examples_fraction.to_f64().unwrap();
	// 					compute_subset_value(example, tree, subset, *left_child_index) * left_weight
	// 						+ compute_subset_value(example, tree, subset, *right_child_index)
	// 							* right_weight
	// 				}
	// 			}
	// 			types::Node::Leaf(types::LeafNode { value, .. }) => value.to_f64().unwrap(),
	// 		}
	// 	}

	// 	struct PowersetIter<T> {
	// 		items: Vec<T>,
	// 		powerset_index: usize,
	// 	}

	// 	impl<T> PowersetIter<T> {
	// 		fn new(items: Vec<T>) -> PowersetIter<T> {
	// 			PowersetIter {
	// 				powerset_index: 0,
	// 				items,
	// 			}
	// 		}
	// 	}

	// 	impl<T: Clone> Iterator for PowersetIter<T> {
	// 		type Item = Vec<T>;
	// 		fn next(&mut self) -> Option<Self::Item> {
	// 			let item = if self.powerset_index == 2usize.pow(self.items.len().to_u32().unwrap()) {
	// 				None
	// 			} else {
	// 				let set = self
	// 					.items
	// 					.iter()
	// 					.enumerate()
	// 					.filter(|&(item_index, _)| (self.powerset_index >> item_index) % 2 == 1)
	// 					.map(|(_, element)| element.clone())
	// 					.collect();
	// 				Some(set)
	// 			};
	// 			self.powerset_index += 1;
	// 			item
	// 		}
	// 	}

	// 	fn factorial(n: usize) -> usize {
	// 		(1..=n).product()
	// 	}
	// }

	// #[test]
	// fn test_shap_simple() {
	// 	let tree = types::Tree {
	// 		nodes: vec![
	// 			types::Node::Branch(types::BranchNode {
	// 				left_child_index: 1,
	// 				right_child_index: 2,
	// 				examples_fraction: 1.0,
	// 				split: types::BranchSplit::Continuous(types::BranchSplitContinuous {
	// 					feature_index: 0,
	// 					split_value: 0.5,
	// 					invalid_values_direction: types::SplitDirection::Left,
	// 				}),
	// 			}),
	// 			types::Node::Leaf(types::LeafNode {
	// 				examples_fraction: 0.5,
	// 				value: 10.0,
	// 			}),
	// 			types::Node::Leaf(types::LeafNode {
	// 				examples_fraction: 0.5,
	// 				value: 100.0,
	// 			}),
	// 		],
	// 	};
	// 	let bias = 0.0;
	// 	let example = &[
	// 		dataframe::Value::Number(1.0),
	// 		dataframe::Value::Number(0.0),
	// 		dataframe::Value::Number(0.0),
	// 	];
	// 	let trees = arr1(&[tree]);
	// 	let shap_values = compute_shap(example, trees.view(), bias);
	// 	let shap_values_brute = brute::compute_shap_brute(example, trees.view(), bias);
	// 	assert_eq!(shap_values, shap_values_brute);
	// }

	// #[test]
	// fn test_shap_duplicate_feature() {
	// 	let tree = types::Tree {
	// 		nodes: vec![
	// 			types::Node::Branch(types::BranchNode {
	// 				left_child_index: 1,
	// 				right_child_index: 2,
	// 				examples_fraction: 1.0,
	// 				split: types::BranchSplit::Continuous(types::BranchSplitContinuous {
	// 					feature_index: 0,
	// 					split_value: 0.5,
	// 					invalid_values_direction: types::SplitDirection::Left,
	// 				}),
	// 			}),
	// 			types::Node::Leaf(types::LeafNode {
	// 				examples_fraction: 0.5,
	// 				value: 10.0,
	// 			}),
	// 			types::Node::Branch(types::BranchNode {
	// 				left_child_index: 3,
	// 				right_child_index: 4,
	// 				examples_fraction: 0.5,
	// 				split: types::BranchSplit::Continuous(types::BranchSplitContinuous {
	// 					feature_index: 0,
	// 					split_value: 0.75,
	// 					invalid_values_direction: types::SplitDirection::Left,
	// 				}),
	// 			}),
	// 			types::Node::Leaf(types::LeafNode {
	// 				examples_fraction: 0.4,
	// 				value: 50.0,
	// 			}),
	// 			types::Node::Leaf(types::LeafNode {
	// 				examples_fraction: 0.1,
	// 				value: 100.0,
	// 			}),
	// 		],
	// 	};
	// 	let bias = 0.0;
	// 	let example = &[
	// 		dataframe::Value::Number(1.0),
	// 		dataframe::Value::Number(0.0),
	// 		dataframe::Value::Number(0.0),
	// 	];
	// 	let trees = arr1(&[tree]);
	// 	let shap_values = compute_shap(example, trees.view(), bias);
	// 	let shap_values_brute = brute::compute_shap_brute(example, trees.view(), bias);
	// 	assert_eq!(shap_values, shap_values_brute);
}
