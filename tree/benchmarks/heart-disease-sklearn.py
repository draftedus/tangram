from sklearn.metrics import accuracy_score, roc_auc_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

from sklearn.experimental import enable_hist_gradient_boosting
from sklearn.ensemble import HistGradientBoostingClassifier

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
model = HistGradientBoostingClassifier(
	learning_rate=0.1,
	max_depth=8,
	max_iter=100,
	max_leaf_nodes=255,
	min_samples_leaf=10,
)
model.fit(features_train, labels_train)

# Compute metrics.
predictions_proba = model.predict_proba(features_test)[:, 1]
auc = roc_auc_score(labels_test, predictions_proba)
print('auc: ', auc)
