/*!The tree module contains functions to train and predict tree models.

There are three model types:

1. [`Regressor`](struct.Regressor.html)
2. [`BinaryClassifier`](struct.BinaryClassifier.html)
3. [`MulticlassClassifier`](struct.MulticlassClassifier.html)

The type of model you are going to train depends on the type of the column you want to predict and in the case of an enum column, the number of unique values.

| Label Type                                      | Number of Enum Options | Model Type                                                 |
|-------------------------------------------------|------------------------|------------------------------------------------------------|
| [Number](../dataframe/struct.NumberColumn.html) | *N/A*                  | [`Regressor`](struct.Regressor.html)                       |
| [Enum](../dataframe/struct.EnumColumn.html)     | 2                      | [`BinaryClassifier`](struct.BinaryClassifier.html)         |
| [Enum](../dataframe/struct.EnumColumn.html)     | > 2                    | [`MulticlassClassifier`](struct.MulticlassClassifier.html) |

Under the hood, Tangram Tree's are Gradient Boosted Decision Trees.

## Training

All models consist of a bias and a series of trees.

### Bias
Training begins by choosing an appropriate bias based on the training dataset and the type of model being trained. The bias for binary classification models is chosen such that the model predicts the "positive" label with probability equal to the percentage of the target column that has the positive label. The bias for multiclass classification models is chosen such that the model predicts the class with the largest representation in the training dataset.

1. **Regression**:  Mean(training_dataset_target_column)
2. **BinaryClassification**:
Log(%training_dataset_target_column==1/%training_dataset_target_column==0)
///TODO: fill in an examples
3. **MulticlassClassification**: Todo
/// Fill in the example of Softmax

### Trees
Begin by measuring the "error" of the naiive model that uses the bias. We then use this error to train the first tree. The goal is to predict the error for each training example.

Each tree is a "regression" tree in that it tries to predict a continous value which is the "error". In order to train a single tree, we need to decide how to split training examples into leaves. The algorithm we use is called "histogram" gradient boosting.

We train trees until we reach the early stopping criteria or the number of trees is equal to max_rounds, whichever comes first.

That's it!

#### Histogram Gradient Boosting
This just means that instead of using the actual raw feature values, we map the values into a discrete number of "bins". This greatly reduces the computational cost of finding optimal splits for the trees and reduces the memory usage significantly. Instead of iterating over `O(n_examples)` to find the optimal split, we only have ot iterate over `O(n_bins)`.
*/

mod bin;
mod binary_classifier;
mod early_stopping;
mod multiclass_classifier;
mod progress;
mod regressor;
mod shap;
mod single;
mod timing;
mod train;
mod types;

pub use self::progress::Progress;
pub use self::types::*;
