<p align="center">
  <img src="tree.svg" title="Tree">
</p>

# Tangram Tree

Tangram tree contains the code for training and making predictions on gradient boosted decision trees.

Tree models consist of a bias and a series of trees.

## Prediction

Before we go through an example to see how we make predictions using tree models, let's cover some tree basics.

In Tangram, a tree model is represented as a list of nodes. Each node is either a `BranchNode` if it is an internal node, or a `LeafNode` if it is a terminal node. A `BranchNode` has a split describing how to decide whether an example should go to the left or right subtree. There are two types of splits: continous splits and categorical splits. If the feature we are using to split examples is a numeric feature, we use a continuous split: all examples with feature value less than or equal to the split threshold value go to the left subtree, and all examples with a feature value greater go to the right subtree. If the feature we are using to split examples is an enum feature, we use a categorical split where the direction we should split based on the enum variant is encoded by the split.

**Continuous Split**:

```rust
pub struct BranchSplitContinuous {
  /// The index of the feature used to split the node.
  pub feature_index: usize,
  /// The threshold value of the split. All features <= split_value go to the left subtree and all features  > split_value go to the right.
  pub split_value: f32,
  /// The subtree (left or right) that invalid values for this feature should go to.
  pub invalid_values_direction: SplitDirection,
}
```

**Discrete Split**:

```rust
pub struct BranchSplitDiscrete {
  /// The index of the feature used to split the node.
  pub feature_index: usize,
  /// The child node direction each enum variant belongs to, 0 for the left child and 1 for the right.
  pub directions: BinDirections,
}
```

Assume we trained a model to predict the price of a home using three features: `number_of_bedrooms`, `total_square_footage`, and `has_garage`.

To make a prediction for a new example:

`y_predict = bias + output_tree_1`

Assume the bias of our model is 256_000.

We would like to predict the price of a house with _4 bedrooms_, _3_200 square feet_ and _no garage_.

Let's start at the root node of tree_1.

**tree_1**

If we had more trees, we would repeat the process we used in determining the output from tree_1, adding the leaf values until we reach the final tree.

The final prediction:

`y_predict= 256_000 + 30_000 - 10_000 = 276_000`.

## Training

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

For more information on contributing, see [CONTRIBUTING.md](CONTRIBUTING.md)
