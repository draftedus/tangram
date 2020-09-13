<p align="center">
  <img src="TangramTree.svg" title="Tangram">
</p>

# Tangram Tree
Tangram tree contains the code for training and making predictions on gradient boosted decision trees.

## Repo Structure

### Before any trees are trained

|file                         | description           |
|-----------------------------|-----------------------|
|**bin.rs**                   | Bin the data.         |
|**regressor.rs**             |This file contains code specific to training and making predictions on regressors. It includes functions that compute biases, gradients and loss for regressors.|
|**binary_classifier.rs**     |This file contains code specific to training and making predictions on binary classifiers. It includes functions that compute biases, gradients, hessians, and loss for binary classifiers.|
|**multiclass_classifier.rs** |This file contains code specific to training and making predictions on multiclass classifiers. It includes functions that compute biases, gradients, hessians and loss for multiclass classifiers.|
|**shap.rs**                  | This module contains code specific to computing [SHAP](https://github.com/slundberg/shap) values.|

### Training a single tree
This code pertains to training a single tree. In the case of multiclass classifiers, one tree is trained per class per round. In the case of binary classifiers and regressors, one tree is trained per round. 

| file                | description|
|---------------------|------------|
|**train.rs**         | This file is the main loop for building a single tree. |
|**bin_stats.rs**     | This contains code that computes aggregate gradient and hessian statistics for all of the examples in a given node. |
| **split.rs**        |  This contains code that finds the optimal split for a node. It uses the bin_stats for the node computed earlier in order to find the optimal split.|
|**examples_index.rs**| This contains code to maintain the examples_index lookup so we keep track of which nodes contain which examples.|
