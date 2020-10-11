from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

# requires that you have java
import h2o
from h2o.estimators import H2OGradientBoostingEstimator
h2o.init()
h2o.no_progress()

# Load the data.
# Load the data.
path = 'data/heart-disease.csv'
nrows_train = 242
nrows_test = 61
target = "diagnosis"
data = pd.read_csv(
	path,
	dtype={
		'age': np.float64,
		'gender': 'category',
		'chest_pain': 'category',
		'resting_blood_pressure': np.float64,
		'cholesterol': np.float64,
		'fasting_blood_sugar_greater_than_120': 'category',
		'resting_ecg_result': 'category',
		'exercise_max_heart_rate': np.float64,
		'exercise_induced_angina': 'category',
		'exercise_st_depression': np.float64,
		'exercise_st_slope': 'category',
		'fluoroscopy_vessels_colored': np.float64,
		'thallium_stress_test': 'category',
		'diagnosis': 'category'
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

# Train the model.
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
