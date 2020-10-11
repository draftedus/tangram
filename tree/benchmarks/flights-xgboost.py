from sklearn.metrics import accuracy_score, roc_auc_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

import xgboost as xgb

# Load the data.
# path_train = 'data/flights-100k.csv'
# path_test = 'data/flights-test.csv'
# nrows_train = 100_000
# nrows_test = 100_000
path_train = 'data/flights-1m.csv'
path_test = 'data/flights-test.csv'
nrows_train = 1_000_000
nrows_test = 100_000
# path_train = 'data/flights-10m.csv'
# path_test = 'data/flights-test.csv'
# nrows_train = 10_000_000
# nrows_test = 100_000
target_column_name = "dep_delayed_15min"
data_train = pd.read_csv(
	path_train,
	dtype={
		'Month': 'object' ,
		'DayofMonth': 'object',
		'DayOfWeek': 'object',
		'DepTime': np.int64,
		'UniqueCarrier': 'object',
		'Origin': 'object',
		'Dest': 'object',
		'Distance': np.int64,
		'dep_delayed_15min': 'object'
	}
).replace({
	'dep_delayed_15min': {
		'N': 0,
		'Y': 1
	}
})
data_test = pd.read_csv(
	path_test,
	dtype={
		'Month': 'object' ,
		'DayofMonth': 'object',
		'DayOfWeek': 'object',
		'DepTime': np.int64,
		'UniqueCarrier': 'object',
		'Origin': 'object',
		'Dest': 'object',
		'Distance': np.int64,
		'dep_delayed_15min': 'object'
	}
).replace({
	'dep_delayed_15min': {
		'N': 0,
		'Y': 1
	}
})
data = pd.get_dummies(pd.concat([data_train, data_test]))
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
model = xgb.XGBClassifier(
	eta = 0.1,
	grow_policy = 'lossguide',
	max_depth = 9,
	n_estimators = 100,
	tree_method = 'hist',
)
model.fit(features_train, labels_train)

# Compute metrics.
predictions_proba = model.predict_proba(features_test)[:, 1]
auc = roc_auc_score(labels_test, predictions_proba)
print('auc: ', auc)