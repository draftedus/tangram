<p align="center">
  <img src="tangram_tree.svg" title="Tangram">
</p>

# Tangram Tree

Tangram tree contains the code for training and making predictions on gradient boosted decision trees.

Tree models consist of a bias and a series of trees.

### Bias

The bias is computed differently depending on the type of model trained: regressor, binary classifier, multiclass classifier. Each of `binary_classifier.rs`, and `multiclass_classifier.rs` have a function called `compute_biases` used to compute biases for that model.

1. **Regressor**: The bias is the mean of values in the target column.
2. **Binary Classifier**: The bias is the logarithm of the ratio of the number of examples in class a and the number of examples in class b. `log(count_class_a / count_class_b)` where `class_a` is the positive class.
3. **Multiclass Classifier**: The bias is a vector of length `n_classes` where the value at index i is the logarithm of the ratio of the number of examples in class_i divided by the total number of examples. `[log(count_class_a / total_count), log(count_class_b/ total_count), log(count_class_c / total_count), ...]`

### Training Trees

- **Step 1.** Measure the _error_ our bias only model generates. In regressor's the loss function is the mean squared error. In binary classifiers the loss function is the binary cross entropy loss, also known as log loss. In multiclass classifiers, the loss function is the cross entropy loss. The code for computing the derivate of the loss function is nested inside a function called `update_gradients_and_hessians` inside each of `regressor.rs`, `binary_classifier.rs`, `multiclass_classifier.rs`.
- **Step 2.** Train a regression tree to predict the error for each example obtained in the previous step. The tree is called a regression tree because the target we are trying to predict for each example is a continuous value. See [Single Tree](single-tree) below for more details.
- **Step 3.** Update the predictions for each example using the output from the previously trained tree. The function to do this is called `update_predictions_from_leaves`, located in `train.rs`. This prediction function is more optimal than running a traditional forward prediction pass for each example through the tree. We are able to do this in a single pass through O(n_examples) because we know which examples ended up in each leaf during training. If we had to predict the naive way, the computational complexity would be O(n_examples)\*O(tree_depth).
- **Step 4.** Update the gradients and hessians for each example, using the predictions and labels.
- **Step 5.** Repeat steps 2 and 3 until we reach the early stopping criteria or we reach `max_rounds`, whichever comes first.

### Single Tree

The goal of training a single tree is to split training examples into leaves such training examples in the same leaves have similar errors. We use the features to split training examples and choose the features that result in the "best" split.

### Histogram Gradient Boosting

Histogram gradient boosting differs from regular gradient boosting in that instead of using the actual raw feature values, we map the values into a discrete number of `bins`. This greatly reduces the computational cost of finding optimal splits for the trees and reduces the memory usage significantly. Instead of iterating over `O(n_examples)` to find the optimal split, we only have ot iterate over `O(n_bins)`.

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
