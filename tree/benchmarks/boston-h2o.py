from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

# requires that you have java
import h2o
from h2o.estimators import H2OGradientBoostingEstimator
h2o.init()
h2o.no_progress()

# load the data
path = 'data/boston.csv'
nrows_train = 405
nrows_test = 101
target = "medv"
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
x = [column for column in data_train.columns if column != target]

# train the model
model = H2OGradientBoostingEstimator(
  distribution="gaussian",
  ntrees = 100,
  max_depth = 8,
  learn_rate = 0.1,
  nbins = 255,
)
model.train(
  training_frame=data_train,
  y=target,
  x=x,
)

# compute mse
perf = model.model_performance(data_test)
print('mse: ', perf.mse())
