from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

# requires that you have java
import h2o
from h2o.estimators import H2OGradientBoostingEstimator
h2o.init()
h2o.no_progress()

# load the data
path = 'data/census.csv'
nrows_train = 26049
nrows_test = 6512
target = "income"
data = pd.read_csv(
	path,
	dtype={
		'age': np.float64,
		'workclass': 'category',
		'fnlwgt': np.float64,
		'education': 'category',
		'education_num': np.float64,
		'marital_status': 'category',
		'occupation': 'category',
		'relationship': 'category',
		'race': 'category',
		'sex': 'category',
		'captial_gain': np.float64,
		'captial_loss': np.float64,
		'hours_per_week': np.float64,
		'native_country': 'category',
		'income': 'category',
	}
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
  distribution="bernoulli",
  ntrees = 100,
  max_depth = 8,
  learn_rate = 0.1,
  nbins = 255
)
model.train(
  training_frame=data_train,
  y=target,
  x=x,
)

# compute accuracy
perf = model.model_performance(data_test)
print('accuracy: ', perf.accuracy()[0][1])

# compute auc
print('auc: ', perf.auc())
