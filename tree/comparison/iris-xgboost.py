from sklearn.metrics import accuracy_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd
import time

import xgboost as xgb

# load the data
path = 'data/iris.csv'
nrows_train = 120
nrows_test = 30
target = "species"
data = pd.read_csv(
	path,
)
features = data.loc[:, data.columns != target]
labels = data[target]
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
	min_child_weight = 100,
	nthread = 24,
	num_round = 100,
	tree_method = 'hist',
)
start = time.time()
model.fit(features_train, labels_train)
end = time.time()
print('xgboost hist')
print('duration: {}ms'.format((end - start)*1000))

# compute accuracy
predictions = model.predict(features_test)
accuracy = accuracy_score(predictions, labels_test)
print('accuracy: ', accuracy)

# train the model
model = xgb.XGBClassifier(
	eta = 0.1,
	grow_policy = 'lossguide',
	max_depth = 8,
	max_leaves = 255,
	min_child_weight = 100,
	nthread = 24,
	num_round = 100,
	tree_method = 'approx'
)
start = time.time()
model.fit(features_train, labels_train)
end = time.time()
print('xgboost non-hist')
print('duration: {}ms'.format((end - start)*1000))

# compute accuracy
predictions = model.predict(features_test)
accuracy = accuracy_score(predictions, labels_test)
print('accuracy: ', accuracy)