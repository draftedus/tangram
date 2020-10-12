from sklearn.metrics import accuracy_score, roc_auc_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd
from pandas.api.types import CategoricalDtype
import argparse

parser = argparse.ArgumentParser()
parser.add_argument('--library', choices=['h2o', 'lightgbm', 'sklearn', 'xgboost'], required=True)
args = parser.parse_args()
library = args.library

# Load the data.
path = 'data/higgs-small.csv'
nrows_train = 450_000
nrows_test = 50_000
# path = 'data/higgs.csv'
# nrows_train = 10_500_000
# nrows_test = 500_000
target_column_name = "signal"
data = pd.read_csv(
	path,
	dtype={
		'signal': np.bool,
		'lepton_pt': np.float64,
		'lepton_eta': np.float64,
		'lepton_phi': np.float64,
		'missing_energy_magnitude': np.float64,
		'missing_energy_phi': np.float64,
		'jet_1_pt': np.float64,
		'jet_1_eta': np.float64,
		'jet_1_phi': np.float64,
		'jet_1_b_tag': np.float64,
		'jet_2_pt': np.float64,
		'jet_2_eta': np.float64,
		'jet_2_phi': np.float64,
		'jet_2_b_tag': np.float64,
		'jet_3_pt': np.float64,
		'jet_3_eta': np.float64,
		'jet_3_phi': np.float64,
		'jet_3_b_tag': np.float64,
		'jet_4_pt': np.float64,
		'jet_4_eta': np.float64,
		'jet_4_phi': np.float64,
		'jet_4_b_tag': np.float64,
		'm_jj': np.float64,
		'm_jjj': np.float64,
		'm_lv': np.float64,
		'm_jlv': np.float64,
		'm_bb': np.float64,
		'm_wbb': np.float64,
		'm_wwbb': np.float64,
	}
)
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
if library == 'h2o':
  import h2o
  from h2o.estimators import H2OGradientBoostingEstimator
  h2o.init()
  data_train = pd.concat([features_train, labels_train], axis=1)
  data_test = pd.concat([features_test, labels_test], axis=1)
  data_train = h2o.H2OFrame(python_obj=data_train)
  data_test = h2o.H2OFrame(python_obj=data_test)
  feature_column_names = [column for column in data_train.columns if column != target_column_name]
  model = H2OGradientBoostingEstimator(
    distribution="bernoulli",
    learn_rate = 0.1,
    max_depth = 9,
    nbins = 255,
    ntrees = 100,
  )
  model.train(
    training_frame=data_train,
    x=feature_column_names,
    y=target_column_name,
  )
elif library == 'lightgbm':
  import lightgbm as lgb
  model = lgb.LGBMClassifier(
    force_row_wise=True,
    learning_rate=0.1,
    n_estimators=100,
    num_leaves=255,
  )
  model.fit(features_train, labels_train)
elif library == 'sklearn':
  from sklearn.experimental import enable_hist_gradient_boosting
  from sklearn.ensemble import HistGradientBoostingClassifier
  model = HistGradientBoostingClassifier(
    learning_rate=0.1,
    max_depth=9,
    max_iter=100,
    max_leaf_nodes=255,
  )
  model.fit(features_train, labels_train)
elif library == 'xgboost':
  import xgboost as xgb
  model = xgb.XGBClassifier(
    eta = 0.1,
    grow_policy = 'lossguide',
    max_depth = 9,
    n_estimators = 100,
    tree_method = 'hist',
  )
  model.fit(features_train, labels_train)

# Compute metrics.
if library == 'h2o':
  predictions_proba = model.predict(data_test).as_data_frame()['True']
else:
  predictions_proba = model.predict_proba(features_test)[:, 1]
auc = roc_auc_score(labels_test, predictions_proba)
print('auc: ', auc)