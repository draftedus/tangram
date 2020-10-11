from sklearn.metrics import accuracy_score, roc_auc_score
from sklearn.model_selection import train_test_split
from time import time
import numpy as np
import pandas as pd

import lightgbm as lgb

# Load the data.
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
features = data.loc[:, data.columns != target]
labels = data[target]
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
	n_estimators=100,
	num_leaves=255,
)
model.fit(
	features_train,
	labels_train
)
print('duration: ', time() - start)

# compute accuracy
predictions = model.predict(features_test)
accuracy = accuracy_score(predictions, labels_test)
print('accuracy: ', accuracy)

# compute auc
predictions_proba = model.predict_proba(features_test)[:, 1]
auc = roc_auc_score(labels_test, predictions_proba)
print('auc: ', auc)
