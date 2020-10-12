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
path = 'data/heart-disease.csv'
nrows_train = 242
nrows_test = 61
target_column_name = "diagnosis"
data = pd.read_csv(
	path,
	dtype={
		'age': np.float64,
		'gender': 'category',
		'chest_pain': 'category',
		'resting_blood_pressure': np.float64,
		'cholesterol': np.float64,
		'fasting_blood_sugar_greater_than_120': 'category',
		'resting_ecg_result': 'category',
		'exercise_max_heart_rate': np.float64,
		'exercise_induced_angina': 'category',
		'exercise_st_depression': np.float64,
		'exercise_st_slope': 'category',
		'fluoroscopy_vessels_colored': np.float64,
		'thallium_stress_test': 'category',
		'diagnosis': 'category'
	}
)
features = data.loc[:, data.columns != target_column_name]
labels = data[target_column_name]

if library == 'xgboost' or library == 'sklearn':
	features = pd.get_dummies(features)

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
    y=target_column_name,
    x=feature_column_names,
  )
elif library == 'lightgbm':
  import lightgbm as lgb
  model = lgb.LGBMClassifier(
    learning_rate=0.1,
    max_depth=9,
    n_estimators=100,
    num_leaves=255,
  )
  model.fit(features_train, labels_train, )
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
