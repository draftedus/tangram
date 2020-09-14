from sklearn.metrics import accuracy_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd
import time

from sklearn.experimental import enable_hist_gradient_boosting
from sklearn.ensemble import HistGradientBoostingClassifier

# load the data
path_train = 'data/flights-10m.csv'
path_test = 'data/flights-test.csv'
target = "dep_delayed_15min"
nrows_train = 10_000_000
nrows_test = 100_000

data_train = pd.read_csv(
  path_train,
  dtype={
    'Month': 'object' ,
    'DayofMonth': 'object',
    'DayOfWeek': 'object',
    'DepTime': np.int64,
    'UniqueCarrier': 'object',
    'Origin': 'object',
    'Dest': 'object',
    'Distance': np.int64,
    'dep_delayed_15min': 'object'
  }
).replace({
  'dep_delayed_15min': {
    'N': 0,
    'Y': 1
  }
})
data_test = pd.read_csv(
  path_test,
  dtype={
    'Month': 'object' ,
    'DayofMonth': 'object',
    'DayOfWeek': 'object',
    'DepTime': np.int64,
    'UniqueCarrier': 'object',
    'Origin': 'object',
    'Dest': 'object',
    'Distance': np.int64,
    'dep_delayed_15min': 'object'
  }
).replace({
  'dep_delayed_15min': {
    'N': 0,
    'Y': 1
  }
})
data = pd.get_dummies(pd.concat([data_train, data_test]))

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
  max_iter=100,
	max_leaf_nodes=512,
)

start = time.time()
model.fit(features_train, labels_train)
end = time.time()
print('duration: {}ms'.format((end-start) * 1000))

# compute accuracy
predictions = model.predict(features_test)
accuracy = accuracy_score(labels_test, predictions)
print('accuracy: ', accuracy)

# compute auc
predictions_proba = model.predict_proba(features_test)[:, 1]
auc = roc_auc_score(labels_test, predictions_proba)
print('auc: ', auc)