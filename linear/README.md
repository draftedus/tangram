<p align="center">
  <img src="tangram_linear.svg" title="Tangram">
</p>

# Tangram Linear

Tangram linear contains the code for training and making predictions on linear models.

Linear models consist of a bias and a series of weights, one for each feature.

`y_predict = bias + weights • feature_values`

where weights and feature_values are both vectors of length n and
• is the dot-product.

## Example

Imagine we are trying to predict the price of a house. We have two features, the number of bedrooms, the total square footage and whether or not the house has a garage.

_Features_: `number_of_bedrooms`, `total_square_feet`, `has_garage`.

During training, our model learned the following weights: `[50_000, 30, 22_000]` and bias: `10_000`.

Let's assume we are a real estate agent who wants to use our formula to assess the price of a new home. The home has _4 bedrooms_, _3,200_ square feet and _no garage_. Using our model:

`y_predict = 10_000 + [50_000, 30, 22_000] • [4, 3_200, 0] = 10_000 + 200_000 + 96_000 + 0 = 306_000`.

| file                     | description                                                                                                 |
| ------------------------ | ----------------------------------------------------------------------------------------------------------- |
| binary_classifier.rs     | This file contains code for training binary classifiers.                                                    |
| multiclass_classifier.rs | This file contains code for training multiclass classifiers.                                                |
| regressor.rs             | This file contains code for training regressors.                                                            |
| shap.rs                  | This file contains code for calculating [SHAP](https://github.com/slundberg/shap) values.                   |
| early_stopping.rs        | This file contains code to evaluate whether training should stop early if the model is no longer improving. |
