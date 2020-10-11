from sklearn.metrics import accuracy_score, roc_auc_score
from sklearn.model_selection import train_test_split
from time import time
import numpy as np
import pandas as pd

import lightgbm as lgb

# Load the data.
# path = 'data/higgs-small.csv'
# nrows_train = 450_000
# nrows_test = 50_000
path = 'data/higgs.csv'
nrows_train = 10_500_000
nrows_test = 500_000
target_column_name = "signal"
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
features = data.loc[:, data.columns != target_column_name]
labels = data[target_column_name]
(features_train, features_test, labels_train, labels_test) = train_test_split(
	features,
	labels,
	test_size=nrows_test,
	train_size=nrows_train,
	shuffle=False
)

# Train the model.
start = time()
model = lgb.LGBMClassifier(
	force_row_wise=True,
	learning_rate=0.1,
	n_estimators=100,
	num_leaves=255,
)
model.fit(features_train, labels_train)
print('duration', time() - start)

# Compute metrics.
predictions_proba = model.predict_proba(features_test)[:, 1]
auc = roc_auc_score(labels_test, predictions_proba)
print('auc: ', auc)
