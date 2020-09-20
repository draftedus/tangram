from sklearn.metrics import mean_squared_error
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

import lightgbm as lgb

# load the data
path = 'data/boston.csv'
nrows_train = 405
nrows_test = 101
target = "medv"
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
model = lgb.LGBMRegressor(
	learning_rate=0.1,
	max_depth=8,
	min_data_in_leaf=100,
	min_sum_hessian_in_leaf=0,
	n_estimators=100,
	num_leaves=255,
	enable_bundle=False,
	enable_sparse=False,
)
model.fit(features_train, labels_train)

# compute mse
predictions = model.predict(features_test)
mse = mean_squared_error(predictions, labels_test)
print('mse: ', mse)
