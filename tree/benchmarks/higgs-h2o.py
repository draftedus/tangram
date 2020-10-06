from sklearn.model_selection import train_test_split
import sys
import numpy as np
import pandas as pd
import json

# requires that you have java
import h2o
from h2o.estimators import H2OGradientBoostingEstimator
h2o.init()

# load the data
# path = 'data/higgs-small.csv'
# nrows_train = 450_000
# nrows_test = 50_000
path = 'data/higgs.csv'
nrows_train = 10_500_000
nrows_test = 500_000
target = "signal"
data = pd.read_csv(
	path,
	dtype={
		'signal': np.bool,
		'lepton_pt': np.float64,
		'lepton_eta': np.float64,
		'lepton_phi': np.float64,
		'missing_energy_magnitude': np.float64,
		'missing_energy_phi': np.float64,
		'jet_1_pt': np.float64,
		'jet_1_eta': np.float64,
		'jet_1_phi': np.float64,
		'jet_1_b_tag': np.float64,
		'jet_2_pt': np.float64,
		'jet_2_eta': np.float64,
		'jet_2_phi': np.float64,
		'jet_2_b_tag': np.float64,
		'jet_3_pt': np.float64,
		'jet_3_eta': np.float64,
		'jet_3_phi': np.float64,
		'jet_3_b_tag': np.float64,
		'jet_4_pt': np.float64,
		'jet_4_eta': np.float64,
		'jet_4_phi': np.float64,
		'jet_4_b_tag': np.float64,
		'm_jj': np.float64,
		'm_jjj': np.float64,
		'm_lv': np.float64,
		'm_jlv': np.float64,
		'm_bb': np.float64,
		'm_wbb': np.float64,
		'm_wwbb': np.float64,
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
  max_depth = 9,
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
