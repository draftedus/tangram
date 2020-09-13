from sklearn.metrics import accuracy_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd
import time

from sklearn.experimental import enable_hist_gradient_boosting
from sklearn.ensemble import HistGradientBoostingClassifier

# load the data
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
model = HistGradientBoostingClassifier(
  learning_rate=0.1,
  max_depth=8,
  max_iter=100,
  max_leaf_nodes=255,
  min_samples_leaf=100,
)
start = time.time()
model.fit(features_train, labels_train)
end = time.time()
print('duration: {}ms'.format((end - start)*1000))

# compute accuracy
predictions = model.predict(features_test)
print('predictions:', model.predict_proba(features_test))
accuracy = accuracy_score(predictions, labels_test)
print('accuracy: ', accuracy)
