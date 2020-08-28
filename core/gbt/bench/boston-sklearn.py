
from sklearn.metrics import mean_squared_error
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd
import time

from sklearn.experimental import enable_hist_gradient_boosting
from sklearn.ensemble import HistGradientBoostingRegressor

# load the data
path = '../data/boston.csv'
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
model = HistGradientBoostingRegressor(
  learning_rate=0.1,
  max_depth=8,
  max_iter=100,
  max_leaf_nodes=255,
  min_samples_leaf=100,
)

start = time.time()
model.fit(features_train, labels_train)
end = time.time()
print('duration: {}ms'.format((end-start) * 1000))

# compute mse
predictions = model.predict(features_test)
mse = mean_squared_error(predictions, labels_test)
print('mse: ', mse)
