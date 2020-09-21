use super::tree::*;

/// This function computes feature importances using the "split" method, where a feature's importance is proportional to the number of nodes that use it to split.
pub fn compute_feature_importances(trees: &[TrainTree], n_features: usize) -> Vec<f32> {
	let mut feature_importances = vec![0.0; n_features];
	for tree in trees.iter() {
		for node in tree.nodes.iter() {
			match node {
				TrainTreeNode::Branch(TrainTreeBranchNode {
					split:
						TrainTreeBranchSplit::Continuous(TrainTreeBranchSplitContinuous {
							feature_index,
							..
						}),
					..
				})
				| TrainTreeNode::Branch(TrainTreeBranchNode {
					split:
						TrainTreeBranchSplit::Discrete(TrainTreeBranchSplitDiscrete {
							feature_index,
							..
						}),
					..
				}) => {
					feature_importances[*feature_index] += 1.0;
				}
				TrainTreeNode::Leaf(_) => {}
			}
		}
	}
	// Normalize the feature_importances.
	let total = feature_importances.iter().sum::<f32>();
	for feature_importance in feature_importances.iter_mut() {
		*feature_importance /= total;
	}
	feature_importances
}
