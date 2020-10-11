from time import time
from sklearn.metrics import accuracy_score, roc_auc_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd

import lightgbm as lgb

# Load the data.
path = 'data/heart-disease.csv'
nrows_train = 242
nrows_test = 61
target_column_name = "diagnosis"
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
	learning_rate=0.1,
	max_depth=8,
	min_child_samples=10,
	min_sum_hessian_in_leaf=0,
	n_estimators=100,
	num_leaves=255,
	enable_bundle=False,
	enable_sparse=False,
)
categorical_feature = ['gender', 'chest_pain', 'fasting_blood_sugar_greater_than_120', 'resting_ecg_result', 'exercise_induced_angina', 'exercise_st_slope', 'thallium_stress_test']
categorical_feature=categorical_feature
model.fit(features_train, labels_train, categorical_feature=categorical_feature)
print(time() - start)

# Compute metrics.
predictions_proba = model.predict_proba(features_test)[:, 1]
auc = roc_auc_score(labels_test, predictions_proba)
print('auc: ', auc)
