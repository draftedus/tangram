<p align="center">
  <img src="tree.svg" title="Tree">
</p>

# Tangram Tree

Tangram tree contains the code for training and making predictions on gradient boosted decision trees.

Tree models consist of a bias and a series of trees. To make a prediction:

`y_predict = bias + output_tree_1 + output_tree_2 + ... + output_tree_n`

## Tree Basics

Before we go through an example prediction, lets cover some tree basics. In Tangram, a tree model is represented as a list of nodes. There are two types of nodes:

- `BranchNode`
- `LeafNode`

A `BranchNode` is an internal tree node and a `LeafNode` is a terminal node. A `BranchNode` has a split describing how to decide whether an example should go to the left or right subtree. There are two types of splits:

- `Continuous`
- `Categorical`

We use `Continuous` splits for numeric features where all examples whose feature value is less than or equal to the threshold go to the left subtree and the rest go to the right. We use `Categorical` splits for enum features where a subset of enum variants should go to the left subtree and the others to the right.

In code:

**`Continuous` Split**:

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

**`Discrete` Split**:

```rust
pub struct BranchSplitDiscrete {
  /// The index of the feature used to split the node.
  pub feature_index: usize,
  /// The child node direction each enum variant belongs to, 0 for the left child and 1 for the right.
  pub directions: BinDirections,
}
```

## Prediction

Imagine we are trying to predict the price of a house. We have three features: the number of bedrooms, the total square footage and whether or not the house has a garage.

_Features_: `number_of_bedrooms`, `square_feet`, `has_garage`.

Let's assume we are a real estate agent who wants to use our tree model to assess the price of a new home. The home has _4 bedrooms_, _3,200_ square feet and _no garage_.

Our trained model consists of 2 trees and a bias. Assume the bias of our model is 256_000. Our final prediction is the sum of the bias and the output of each of the trees.

<p align="center">
  <img src="trees.svg" title="Tree">
</p>

Let's start at the root node of `tree_1`. We have a `Continuous` split on the `square_feet` feature, where are examples who's square footage is less than or equal to 4,000 should go left. Our house has _3,200_ square feet so we go left. The next node is a `Categorical` split on whether our home has a garage. Homes with a garage go to the right and homes without go to the left. Now we have reached a leaf node and we update our prediction with this value:

`y_current_prediction = 256_000 + 30_108 +...`

We repeat this process with the second tree, using the splits to determine our path to a leaf node. When we reach the final tree, we are done and we are left with the prediction:

`y_predict= 256_000 + 30_108 - 1_304 = 284_804`.

We just went through a simple example of how to make predictions with a Tree Regressor.

### Predicting Binary Classifier Outputs

How do we use this method to make predictions with binary classifiers? Each leaf's value is now an intermediate value called a _logit_. To make our final prediction, we take the sum of the bias and outputs of each tree and pass it through the sigmoid function:

`y_predict = sigmoid(bias + output_tree_1 + output_tree_2 + ...)`

You can think of the sigmoid function as a special function that squashes values that range from -infinity to infinity to values that range from 0 to 1.

### Predicting Multiclass Classifier Outputs

Instead of training a single tree per round, we now train n trees per round, one for each class. To make a prediction for a model with `n` classes and `m` rounds:

<p align="center">
  <img src="multiclass.svg" title="Tree">
</p>

```
y_predict = softmax(
  [bias_class_1, bias_class_2, ..., bias_class_n] +
  [output_round_1_tree_class_1, output_round_1_tree_class_2, ..., output_round_1_tree_class_n] +
  [output_round_2_tree_class_1, output_round_2_tree_class_2, ..., output_round_2_tree_class_n] +
  [output_round_m_tree_class_1, output_round_m_tree_class_2, ..., output_round_m_tree_class_n]
)

```

## Training

We saw how to make a prediction using an already trained tree. Let's learn a little bit about how trees are trained. This description should be supplemented by walking through the code which we have done our best to document and make easy to follow.

### Bias

The bias is computed differently depending on the type of model trained: regressor, binary classifier, multiclass classifier. Each of `binary_classifier.rs`, and `multiclass_classifier.rs` have a function called `compute_biases` used to compute biases for that model.

1. **Regressor**: The bias is the mean of values in the target column.
2. **Binary Classifier**: The bias is the logarithm of the ratio of the number of examples in class a and the number of examples in class b. `log(count_class_a / count_class_b)` where `class_a` is the positive class.
3. **Multiclass Classifier**: The bias is a vector of length `n_classes` where the value at index i is the logarithm of the ratio of the number of examples in class_i divided by the total number of examples. `[log(count_class_a / total_count), log(count_class_b/ total_count), log(count_class_c / total_count), ...]`

### Training Trees

- **Step 1.** Measure the _error_ our bias only model generates. In regressor's the loss function is the mean squared error. In binary classifiers the loss function is the binary cross entropy loss, also known as log loss. In multiclass classifiers, the loss function is the cross entropy loss. The code for computing the derivate of the loss function is nested inside a function called `update_gradients_and_hessians` inside each of `regressor.rs`, `binary_classifier.rs`, `multiclass_classifier.rs`.
- **Step 2.** Train a regression tree to predict the error for each example obtained in the previous step. The tree is called a regression tree because the target we are trying to predict for each example is a continuous value. See [Single Tree](#single-tree) below for more details.
- **Step 3.** Update the predictions for each example using the output from the previously trained tree.
- **Step 4.** Update the gradients and hessians for each example, using the predictions and labels.
- **Step 5.** Repeat steps 2 and 3 until we reach the early stopping criteria or we reach `max_rounds`, whichever comes first.

### Single Tree

The goal of training a single tree is to split training examples into leaves such training examples in the same leaves have similar errors. We use the features to split training examples and choose the features that result in the "best" split. See the `train` function in `single.rs`.

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
| **single.rs**                | This file is the main loop for building a single tree.                                                                                                                                             |
| **bin_stats.rs**             | This contains code that computes aggregate gradient and hessian statistics for all of the examples in a given node.                                                                                |
| **split.rs**                 | This contains code that finds the optimal split for a node. It uses the bin_stats for the node computed earlier in order to find the optimal split.                                                |
| **examples_index.rs**        | This contains code to maintain the examples_index lookup so we keep track of which nodes contain which examples.                                                                                   |

```

```
