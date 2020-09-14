# Contributing

Thank you for your interest in making improvements to Tangram Tree.

## Repo Structure

### Main Files

| file                         | description                                                                                                                                                                                        |
| ---------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **bin.rs**                   | Bin the data.                                                                                                                                                                                      |
| **regressor.rs**             | This file contains code specific to training and making predictions on regressors. It includes functions that compute biases, gradients and loss for regressors.                                   |
| **binary_classifier.rs**     | This file contains code specific to training and making predictions on binary classifiers. It includes functions that compute biases, gradients, hessians, and loss for binary classifiers.        |
| **multiclass_classifier.rs** | This file contains code specific to training and making predictions on multiclass classifiers. It includes functions that compute biases, gradients, hessians and loss for multiclass classifiers. |
| **shap.rs**                  | This module contains code specific to computing [SHAP](https://github.com/slundberg/shap) values.                                                                                                  |

### Training a single tree

All code under `single` pertains to training a single tree. In the case of multiclass classifiers, one tree is trained per class per round. In the case of binary classifiers and regressors, one tree is trained per round. A single tree is trained using a priority queue where the branch with the current best score is evaluated first. Alternatively, you could train a tree breadth first. At each iteration of training, we first compute the bin stats for the node. The code for this is located in `bin_stats.rs`.Computing bin stats is the most computationally expensive step in training trees. Once the bin stats are computed, we find the best split. The code for finding the optimal split is located in `split.rs`. If there is a split that is valid, we split the node and push the two newly created nodes into the queue. If there is no valid split, we make the node a leaf node. Once we have split a node, we need to reorder the examples_index lookup table such that all examples that go to the left subtree are contiguous and all examples that go the right are contiguous. The code to reorder to examples_index is in `examples_index.rs`.

| file                  | description                                                                                                                                         |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| **train.rs**          | This file is the main loop for building a single tree.                                                                                              |
| **bin_stats.rs**      | This contains code that computes aggregate gradient and hessian statistics for all of the examples in a given node.                                 |
| **split.rs**          | This contains code that finds the optimal split for a node. It uses the bin_stats for the node computed earlier in order to find the optimal split. |
| **examples_index.rs** | This contains code to maintain the examples_index lookup so we keep track of which nodes contain which examples.                                    |
