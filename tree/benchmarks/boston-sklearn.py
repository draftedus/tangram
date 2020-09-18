
from sklearn.metrics import mean_squared_error
from sklearn.model_selection import train_test_split
from sklearn.dummy import DummyRegressor
import numpy as np
import pandas as pd
import time

from sklearn.experimental import enable_hist_gradient_boosting
from sklearn.ensemble import HistGradientBoostingRegressor

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
model = HistGradientBoostingRegressor()

start = time.time()
model.fit(features_train, labels_train)
end = time.time()
print('duration: {}ms'.format((end-start) * 1000))

# compute mse
predictions = model.predict(features_test)
mse = mean_squared_error(predictions, labels_test)
print('mse: ', mse)

# compute baseline
baseline_model = DummyRegressor()
baseline_model.fit(features_train, labels_train)
baseline_predictions = baseline_model.predict(features_test)
baseline_mse = mean_squared_error(baseline_predictions, labels_test)
print('baseline_mse: ', baseline_mse)