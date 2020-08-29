from sklearn.metrics import accuracy_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd
import time

import lightgbm as lgb

# load the data
path = '../data/heart-disease.csv'
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
start = time.time()
categorical_feature = ['gender', 'chest_pain', 'fasting_blood_sugar_greater_than_120', 'resting_ecg_result', 'exercise_induced_angina', 'exercise_st_slope', 'thallium_stress_test']
categorical_feature=categorical_feature
model.fit(features_train, labels_train, categorical_feature=categorical_feature)
end = time.time()
print('duration: {}ms'.format((end-start) * 1000))

# compute accuracy
predictions = model.predict(features_test)
accuracy = accuracy_score(predictions, labels_test)
print('accuracy: ', accuracy)
