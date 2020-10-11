from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

import h2o
from h2o.estimators import H2OGradientBoostingEstimator
h2o.init()

# Load the data.
path = 'data/boston.csv'
nrows_train = 405
nrows_test = 101
target_column_name = "medv"
data = pd.read_csv(
	path,
)
(data_train, data_test) = train_test_split(
	data,
	test_size=nrows_test,
	train_size=nrows_train,
	shuffle=False
)
data_train = h2o.H2OFrame(python_obj=data_train)
data_test = h2o.H2OFrame(python_obj=data_test)
feature_column_names = [column for column in data_train.columns if column != target_column_name]

# Train the model.
model = H2OGradientBoostingEstimator(
  distribution="gaussian",
  learn_rate = 0.1,
  nbins = 255,
  ntrees = 100,
)
model.train(
  training_frame=data_train,
  y=target_column_name,
  x=feature_column_names,
)

# Compute metrics.
perf = model.model_performance(data_test)
print('mse: ', perf.mse())
