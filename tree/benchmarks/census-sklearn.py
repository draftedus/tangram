from sklearn.metrics import accuracy_score, roc_auc_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

from sklearn.experimental import enable_hist_gradient_boosting
from sklearn.ensemble import HistGradientBoostingClassifier

# Load the data.
path = 'data/census.csv'
nrows_train = 26049
nrows_test = 6512
target = "income"
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

# Train the model.
model = HistGradientBoostingClassifier(
	learning_rate=0.1,
	max_depth=8,
	max_iter=100,
	max_leaf_nodes=255,
	min_samples_leaf=100,
)
model.fit(features_train, labels_train)

# compute accuracy
predictions = model.predict(features_test)
accuracy = accuracy_score(predictions, labels_test)
print('accuracy: ', accuracy)

# compute auc
predictions_proba = model.predict_proba(features_test)[:, 1]
auc = roc_auc_score(labels_test, predictions_proba)
print('auc: ', auc)
