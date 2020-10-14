from sklearn.metrics import mean_squared_error
from sklearn.model_selection import train_test_split
import argparse
import numpy as np
import pandas as pd

parser = argparse.ArgumentParser()
parser.add_argument('--library', choices=['h2o', 'lightgbm', 'sklearn', 'xgboost'], required=True)
args = parser.parse_args()

# Load the data.
path = 'data/boston.csv'
nrows_train = 405
nrows_test = 101
target_column_name = "medv"
data = pd.read_csv(path)
features = data.loc[:, data.columns != target_column_name]
labels = data[target_column_name]
(features_train, features_test, labels_train, labels_test) = train_test_split(
	features,
	labels,
	test_size=nrows_test,
	train_size=nrows_train,
	shuffle=False
)

# Train the model.
if args.library == 'h2o':
  import h2o
  from h2o.estimators import H2OGradientBoostingEstimator
  h2o.init()
  data_train = pd.concat([features_train, labels_train], axis=1)
  data_test = pd.concat([features_test, labels_test], axis=1)
  data_train = h2o.H2OFrame(python_obj=data_train)
  data_test = h2o.H2OFrame(python_obj=data_test)
  feature_column_names = [column for column in data_train.columns if column != target_column_name]
  model = H2OGradientBoostingEstimator(
    distribution="gaussian",
    learn_rate=0.1,
    ntrees=100,
  )
  model.train(
    training_frame=data_train,
    y=target_column_name,
    x=feature_column_names,
  )
elif args.library == 'lightgbm':
  import lightgbm as lgb
  model = lgb.LGBMRegressor(
    learning_rate=0.1,
    n_estimators=100,
    num_leaves=255,
  )
  model.fit(features_train, labels_train)
elif args.library == 'sklearn':
  from sklearn.experimental import enable_hist_gradient_boosting
  from sklearn.ensemble import HistGradientBoostingRegressor
  model = HistGradientBoostingRegressor(
    learning_rate=0.1,
    max_iter=100,
  	max_leaf_nodes=255,
  )
  model.fit(features_train, labels_train)
elif args.library == 'xgboost':
  import xgboost as xgb
  model = xgb.XGBRegressor(
    eta=0.1,
    grow_policy='lossguide',
    max_leaves=255,
    n_estimators=100,
    tree_method='hist',
  )
  model.fit(features_train, labels_train)

# Make predictions on the test data.
if args.library == 'h2o':
  predictions = model.predict(data_test).as_data_frame()
else:
  predictions = model.predict(features_test)

# Compute metrics.
mse = mean_squared_error(predictions, labels_test)
print('mse', mse)
