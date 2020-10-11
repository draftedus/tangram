from sklearn.metrics import accuracy_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

import lightgbm as lgb

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
model = lgb.LGBMClassifier(
	learning_rate=0.1,
	max_depth=9,
	n_estimators=100,
	num_leaves=255,
)
model.fit(features_train, labels_train)

# Compute metrics.
predictions = model.predict(features_test)
accuracy = accuracy_score(predictions, labels_test)
print('accuracy: ', accuracy)
