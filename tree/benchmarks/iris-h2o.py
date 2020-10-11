from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

# requires that you have java
import h2o
from h2o.estimators import H2OGradientBoostingEstimator
h2o.init()

# Load the data.
path = 'data/iris.csv'
nrows_train = 120
nrows_test = 30
target_column_name = "species"
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
  learn_rate = 0.1,
  max_depth = 9,
  nbins = 255
  ntrees = 100,
)
model.train(
  training_frame=data_train,
  x=feature_column_names,
  y=target_column_name,
)

# Compute metrics.
perf = model.model_performance(data_test)
print('accuracy: ', 1 - perf.confusion_matrix()[3][3])
