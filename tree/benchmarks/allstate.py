from pandas.api.types import CategoricalDtype
from sklearn.metrics import mean_squared_error, mean_absolute_error
import argparse
import numpy as np
import pandas as pd
import json

parser = argparse.ArgumentParser()
parser.add_argument('--library', choices=['h2o', 'lightgbm', 'sklearn', 'xgboost'], required=True)
args = parser.parse_args()

# Load the data.
path_train = 'data/allstate_train.csv'
path_test = 'data/allstate_test.csv'
target_column_name = "claim_amount"
blind_make_options = ['AU', 'BF', 'AR', 'AJ', 'BO', 'BW', 'BH', 'AQ', 'L', 'BP', 'AN', 'K', 'AO', 'AH', 'D', 'X', 'Y', 'W', 'BU', 'Q', 'R', 'AL', 'BV', 'M', 'I', 'BG', 'BT', 'E', 'S', 'AY', 'P', 'N', 'O', 'AI', 'Z', 'BZ', 'BY', 'BM', 'AX', 'J', 'BN', 'BS', 'AZ', 'BB', 'AV', 'BD','AF', 'G', 'AC', 'AW', '?', 'BR', 'BA', 'V', 'AD', 'AE', 'B', 'U','AP', 'AM', 'BC', 'CB', 'AT', 'BL', 'F', 'AG', 'A', 'AS', 'BQ', 'AK', 'CA', 'BK', 'H', 'C', 'AB']
blind_model_options = ['AU.54', 'BF.36', 'AR.41']
blind_submodel_options = ['AU.54.3','BF.36.3']
cat1_options = ['G', 'B', 'D', 'I', 'F', 'A', 'E', 'C', 'H', 'J', '?']
cat2_options = ['B', 'C', '?', 'A']
cat3_options = ['A', 'F', 'B', 'C', 'E', 'D', '?']
cat4_options = ['A', '?', 'C', 'B']
cat5_options = ['C', 'A', '?', 'B']
cat6_options = ['D', 'C', 'E', 'B', 'F', '?']
cat7_options = ['A', 'C', '?', 'D', 'B']
cat8_options = ['A', 'B', 'C', '?']
cat9_options = ['B', 'A']
cat10_options = ['A', 'B', 'C', '?']
cat11_options = ['B', 'E', 'C', 'A', 'D', 'F', '?']
cat12_options = ['C', 'B', 'E', 'D', 'F', 'A']
ordcat_options = ['4', '2', '5', '3', '7', '6', '1', '?']
nvcat_options = ['L', 'N', 'M', 'J', 'B', 'E', 'O', 'F', 'K', 'D', 'A', 'H', 'C', 'G', 'I']
dtype = {
  'row_id': np.int64,
  'household_id': np.int64,
  'vehicle': np.int64,
  'calendar_year': np.int64,
  'model_year': np.int64,
  'blind_make': CategoricalDtype(categories=blind_make_options),
  'blind_model': CategoricalDtype(categories=blind_model_options),
  'blind_submodel': CategoricalDtype(categories=blind_submodel_options),
  'cat1': CategoricalDtype(categories=cat1_options),
  'cat2': CategoricalDtype(categories=cat2_options),
  'cat3': CategoricalDtype(categories=cat3_options),
  'cat4': CategoricalDtype(categories=cat4_options),
  'cat5': CategoricalDtype(categories=cat5_options),
  'cat6': CategoricalDtype(categories=cat6_options),
  'cat7': CategoricalDtype(categories=cat7_options),
  'cat8': CategoricalDtype(categories=cat8_options),
  'cat9': CategoricalDtype(categories=cat9_options),
  'cat10': CategoricalDtype(categories=cat10_options),
  'cat11': CategoricalDtype(categories=cat11_options),
  'cat12': CategoricalDtype(categories=cat12_options),
  'ordcat': CategoricalDtype(categories=ordcat_options),
  'var1': np.float64,
  'var2': np.float64,
  'var3':np.float64,
  'var4': np.float64,
  'var5': np.float64,
  'var6': np.float64,
  'var7': np.float64,
  'var8': np.float64,
  'nvcat': CategoricalDtype(categories=nvcat_options),
  'nvvar2': np.float64,
  'nvvar3': np.float64,
  'nvvar4': np.float64,
  'claim_amount': np.float64,
}
data_train = pd.read_csv(path_train, dtype=dtype)
data_test = pd.read_csv(path_test, dtype=dtype)
features_train = data_train.loc[:, data_train.columns != target_column_name]
labels_train = data_train[target_column_name]
features_test = data_test.loc[:, data_test.columns != target_column_name]
labels_test = data_test[target_column_name]

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
mae = mean_absolute_error(predictions, labels_test)
print(json.dumps({
  'mse': mse,
  'mae': mae
}))
