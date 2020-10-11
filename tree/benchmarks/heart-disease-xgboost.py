from sklearn.metrics import accuracy_score, roc_auc_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

import xgboost as xgb

# Load the data.
path = 'data/heart-disease.csv'
nrows_train = 242
nrows_test = 61
target_column_name = "diagnosis"
data = pd.read_csv(
	path,
)
features = data.loc[:, data.columns != target_column_name]
labels = data[target_column_name]
features = pd.get_dummies(features)
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