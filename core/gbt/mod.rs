/*!The tree module contains functions to train and predict tree models.

There are three model types:

1. [`Regressor`](struct.Regressor.html)
2. [`BinaryClassifier`](struct.BinaryClassifier.html)
3. [`MulticlassClassifier`](struct.MulticlassClassifier.html)

The type of model you are going to train depends on the type of the column you want to predict and in the case of an enum column, the number of unique values.

| Label Type                                      | Number of Enum Options | Model Type                                                 |
|-------------------------------------------------|------------------------|------------------------------------------------------------|
| [Number](../dataframe/struct.NumberColumn.html) | N/A                    | [`Regressor`](struct.Regressor.html)                       |
| [Enum](../dataframe/struct.EnumColumn.html)     | 2                      | [`BinaryClassifier`](struct.BinaryClassifier.html)         |
| [Enum](../dataframe/struct.EnumColumn.html)     | >2                     | [`MulticlassClassifier`](struct.MulticlassClassifier.html) |

*/

mod bin;
mod binary_classifier;
mod early_stopping;
mod multiclass_classifier;
mod progress;
mod regressor;
mod shap;
// mod timing;
mod train;
mod tree;
mod types;

pub use self::progress::Progress;
pub use self::types::*;
