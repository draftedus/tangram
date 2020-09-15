use crate::single;

/// This function computes feature importances using the "split" method, where a feature's importance is proportional to the number of nodes that use it to split.
pub fn compute_feature_importances(trees: &[single::TrainTree], n_features: usize) -> Vec<f32> {
	let mut feature_importances = vec![0.0; n_features];
	for tree in trees.iter() {
		tree.nodes.iter().for_each(|node| match node {
			single::TrainNode::Branch(single::TrainBranchNode {
				split:
					single::TrainBranchSplit::Continuous(single::TrainBranchSplitContinuous {
						feature_index,
						..
					}),
				..
			})
			| single::TrainNode::Branch(single::TrainBranchNode {
				split:
					single::TrainBranchSplit::Discrete(single::TrainBranchSplitDiscrete {
						feature_index,
						..
					}),
				..
			}) => {
				feature_importances[*feature_index] += 1.0;
			}
			single::TrainNode::Leaf(_) => {}
		});
	}
	// Normalize the feature_importances.
	let total: f32 = feature_importances.iter().sum();
	for feature_importance in feature_importances.iter_mut() {
		*feature_importance /= total;
	}
	feature_importances
}
