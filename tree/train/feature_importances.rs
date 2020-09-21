use super::tree::*;

/// This function computes feature importances using the "split" method, where a feature's importance is proportional to the number of nodes that use it to split.
pub fn compute_feature_importances(trees: &[TrainTree], n_features: usize) -> Vec<f32> {
	let mut feature_importances = vec![0.0; n_features];
	for tree in trees.iter() {
		for node in tree.nodes.iter() {
			match node {
				TrainNode::Branch(TrainBranchNode {
					split:
						TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
							feature_index, ..
						}),
					..
				})
				| TrainNode::Branch(TrainBranchNode {
					split:
						TrainBranchSplit::Discrete(TrainBranchSplitDiscrete { feature_index, .. }),
					..
				}) => {
					feature_importances[*feature_index] += 1.0;
				}
				TrainNode::Leaf(_) => {}
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
