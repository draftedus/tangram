from sklearn.metrics import accuracy_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

from sklearn.experimental import enable_hist_gradient_boosting
from sklearn.ensemble import HistGradientBoostingClassifier

# Load the data.
path = 'data/iris.csv'
nrows_train = 120
nrows_test = 30
target_column_name = "species"
data = pd.read_csv(
	path,
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
model = HistGradientBoostingClassifier(
	learning_rate=0.1,
	max_depth=9,
	max_iter=100,
	max_leaf_nodes=255,
	min_samples_leaf=1,
)
model.fit(features_train, labels_train)

# Compute metrics.
predictions = model.predict(features_test)
accuracy = accuracy_score(predictions, labels_test)
print('accuracy: ', accuracy)
