from sklearn.metrics import accuracy_score, roc_auc_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

import xgboost as xgb

# load the data
path = 'data/heart-disease.csv'
nrows_train = 242
nrows_test = 61
target = "diagnosis"
data = pd.read_csv(
	path,
)
features = data.loc[:, data.columns != target]
labels = data[target]
features = pd.get_dummies(features)
(features_train, features_test, labels_train, labels_test) = train_test_split(
	features,
	labels,
	test_size=nrows_test,
	train_size=nrows_train,
	shuffle=False
)

# train the model
model = xgb.XGBClassifier(
	eta = 0.1,
	grow_policy = 'lossguide',
	max_depth = 8,
	max_leaves = 255,
	min_child_weight = 10,
	num_round = 100,
	tree_method = 'hist',
)
model.fit(features_train, labels_train)

# compute accuracy
predictions = model.predict(features_test)
accuracy = accuracy_score(labels_test, predictions)
print('accuracy: ', accuracy)

# compute auc
predictions_proba = model.predict_proba(features_test)[:, 1]
auc = roc_auc_score(labels_test, predictions_proba)
print('auc: ', auc)