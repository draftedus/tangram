from sklearn.metrics import accuracy_score, roc_auc_score
from sklearn.model_selection import train_test_split
import numpy as np
import pandas as pd
from pandas.api.types import CategoricalDtype
import argparse

parser = argparse.ArgumentParser()
parser.add_argument('--library', choices=['h2o', 'lightgbm', 'sklearn', 'xgboost'], required=True)
args = parser.parse_args()
library = args.library

# Load the data.
path_train = 'data/flights-100k.csv'
path_test = 'data/flights-test.csv'
nrows_train=100_000
nrows_test=100_000
# path_train = 'data/flights-1m.csv'
# path_test = 'data/flights-test.csv'
# nrows_train=1_000_000
# nrows_test=100_000
# path_test = 'data/flights-test.csv'
# nrows_train=10_000_000
# nrows_test=100_000
month_options = [
	"c-1", "c-10", "c-11", "c-12", "c-2", "c-3", "c-4", "c-5", "c-6", "c-7", "c-8", "c-9",
]
day_of_week_options = ["c-1", "c-2", "c-3", "c-4", "c-5", "c-6", "c-7"]
day_of_month_options = [
	"c-1", "c-10", "c-11", "c-12", "c-13", "c-14", "c-15", "c-16", "c-17", "c-18", "c-19",
	"c-2", "c-20", "c-21", "c-22", "c-23", "c-24", "c-25", "c-26", "c-27", "c-28", "c-29",
	"c-3", "c-30", "c-31", "c-4", "c-5", "c-6", "c-7", "c-8", "c-9",
]
carrier_options = [
	"AA", "AQ", "AS", "B6", "CO", "DH", "DL", "EV", "F9", "FL", "HA", "HP", "MQ", "NW", "OH",
	"OO", "TZ", "UA", "US", "WN", "XE", "YV",
]
origin_options = [
	"ABE", "ABI", "ABQ", "ABY", "ACK", "ACT", "ACV", "ACY", "ADK", "ADQ", "AEX", "AGS", "AKN",
	"ALB", "AMA", "ANC", "APF", "ASE", "ATL", "ATW", "AUS", "AVL", "AVP", "AZO", "BDL", "BET",
	"BFL", "BGM", "BGR", "BHM", "BIL", "BIS", "BLI", "BMI", "BNA", "BOI", "BOS", "BPT", "BQK",
	"BQN", "BRO", "BRW", "BTM", "BTR", "BTV", "BUF", "BUR", "BWI", "BZN", "CAE", "CAK", "CDC",
	"CDV", "CEC", "CHA", "CHO", "CHS", "CIC", "CID", "CLD", "CLE", "CLL", "CLT", "CMH", "CMI",
	"COD", "COS", "CPR", "CRP", "CRW", "CSG", "CVG", "CWA", "DAB", "DAL", "DAY", "DBQ", "DCA",
	"DEN", "DFW", "DHN", "DLG", "DLH", "DRO", "DSM", "DTW", "EGE", "EKO", "ELP", "ERI", "EUG",
	"EVV", "EWR", "EYW", "FAI", "FAR", "FAT", "FAY", "FCA", "FLG", "FLL", "FLO", "FNT", "FSD",
	"FSM", "FWA", "GEG", "GFK", "GGG", "GJT", "GNV", "GPT", "GRB", "GRK", "GRR", "GSO", "GSP",
	"GST", "GTF", "GTR", "GUC", "HDN", "HKY", "HLN", "HNL", "HOU", "HPN", "HRL", "HSV", "HTS",
	"HVN", "IAD", "IAH", "ICT", "IDA", "ILG", "ILM", "IND", "IPL", "ISO", "ISP", "ITO", "IYK",
	"JAC", "JAN", "JAX", "JFK", "JNU", "KOA", "KTN", "LAN", "LAS", "LAW", "LAX", "LBB", "LCH",
	"LEX", "LFT", "LGA", "LGB", "LIH", "LIT", "LNK", "LRD", "LSE", "LWB", "LWS", "LYH", "MAF",
	"MBS", "MCI", "MCN", "MCO", "MDT", "MDW", "MEI", "MEM", "MFE", "MFR", "MGM", "MHT", "MIA",
	"MKE", "MLB", "MLI", "MLU", "MOB", "MOD", "MOT", "MQT", "MRY", "MSN", "MSO", "MSP", "MSY",
	"MTJ", "MYR", "OAJ", "OAK", "OGG", "OKC", "OMA", "OME", "ONT", "ORD", "ORF", "OTZ", "OXR",
	"PBI", "PDX", "PFN", "PHF", "PHL", "PHX", "PIA", "PIE", "PIH", "PIT", "PNS", "PSC", "PSE",
	"PSG", "PSP", "PVD", "PWM", "RAP", "RDD", "RDM", "RDU", "RFD", "RIC", "RNO", "ROA", "ROC",
	"RST", "RSW", "SAN", "SAT", "SAV", "SBA", "SBN", "SBP", "SCC", "SCE", "SDF", "SEA", "SFO",
	"SGF", "SGU", "SHV", "SIT", "SJC", "SJT", "SJU", "SLC", "SMF", "SMX", "SNA", "SOP", "SPI",
	"SPS", "SRQ", "STL", "STT", "STX", "SUN", "SWF", "SYR", "TEX", "TLH", "TOL", "TPA", "TRI",
	"TTN", "TUL", "TUP", "TUS", "TVC", "TWF", "TXK", "TYR", "TYS", "VCT", "VIS", "VLD", "VPS",
	"WRG", "WYS", "XNA", "YAK", "YUM",
]
dest_options= [
	"ABE", "ABI", "ABQ", "ABY", "ACK", "ACT", "ACV", "ACY", "ADK", "ADQ", "AEX", "AGS", "AKN",
	"ALB", "AMA", "ANC", "APF", "ASE", "ATL", "ATW", "AUS", "AVL", "AVP", "AZO", "BDL", "BET",
	"BFL", "BGM", "BGR", "BHM", "BIL", "BIS", "BLI", "BMI", "BNA", "BOI", "BOS", "BPT", "BQK",
	"BQN", "BRO", "BRW", "BTM", "BTR", "BTV", "BUF", "BUR", "BWI", "BZN", "CAE", "CAK", "CDC",
	"CDV", "CEC", "CHA", "CHO", "CHS", "CIC", "CID", "CLD", "CLE", "CLL", "CLT", "CMH", "CMI",
	"COD", "COS", "CPR", "CRP", "CRW", "CSG", "CVG", "CWA", "DAB", "DAL", "DAY", "DBQ", "DCA",
	"DEN", "DFW", "DHN", "DLG", "DLH", "DRO", "DSM", "DTW", "EGE", "EKO", "ELP", "ERI", "EUG",
	"EVV", "EWR", "EYW", "FAI", "FAR", "FAT", "FAY", "FCA", "FLG", "FLL", "FLO", "FNT", "FSD",
	"FSM", "FWA", "GEG", "GFK", "GGG", "GJT", "GNV", "GPT", "GRB", "GRK", "GRR", "GSO", "GSP",
	"GST", "GTF", "GTR", "GUC", "HDN", "HKY", "HLN", "HNL", "HOU", "HPN", "HRL", "HSV", "HTS",
	"HVN", "IAD", "IAH", "ICT", "IDA", "ILG", "ILM", "IND", "IPL", "ISO", "ISP", "ITO", "IYK",
	"JAC", "JAN", "JAX", "JFK", "JNU", "KOA", "KTN", "LAN", "LAS", "LAW", "LAX", "LBB", "LBF",
	"LCH", "LEX", "LFT", "LGA", "LGB", "LIH", "LIT", "LNK", "LRD", "LSE", "LWB", "LWS", "LYH",
	"MAF", "MBS", "MCI", "MCN", "MCO", "MDT", "MDW", "MEI", "MEM", "MFE", "MFR", "MGM", "MHT",
	"MIA", "MKE", "MLB", "MLI", "MLU", "MOB", "MOD", "MOT", "MQT", "MRY", "MSN", "MSO", "MSP",
	"MSY", "MTJ", "MYR", "OAJ", "OAK", "OGG", "OKC", "OMA", "OME", "ONT", "ORD", "ORF", "OTZ",
	"OXR", "PBI", "PDX", "PFN", "PHF", "PHL", "PHX", "PIA", "PIE", "PIH", "PIT", "PNS", "PSC",
	"PSE", "PSG", "PSP", "PVD", "PWM", "RAP", "RDD", "RDM", "RDU", "RFD", "RIC", "RNO", "ROA",
	"ROC", "RST", "RSW", "SAN", "SAT", "SAV", "SBA", "SBN", "SBP", "SCC", "SCE", "SDF", "SEA",
	"SFO", "SGF", "SGU", "SHV", "SIT", "SJC", "SJT", "SJU", "SLC", "SMF", "SMX", "SNA", "SOP",
	"SPI", "SPS", "SRQ", "STL", "STT", "STX", "SUN", "SWF", "SYR", "TEX", "TLH", "TOL", "TPA",
	"TRI", "TTN", "TUL", "TUP", "TUS", "TVC", "TWF", "TXK", "TYR", "TYS", "VCT", "VIS", "VLD",
	"VPS", "WRG", "WYS", "XNA", "YAK", "YUM",
]
data_train = pd.read_csv(
	path_train,
	dtype={
		'Month': CategoricalDtype(categories=month_options) ,
		'DayofMonth': CategoricalDtype(categories=day_of_month_options),
		'DayOfWeek': CategoricalDtype(categories=day_of_week_options),
		'DepTime': np.int64,
		'UniqueCarrier': CategoricalDtype(categories=carrier_options),
		'Origin': CategoricalDtype(categories=origin_options),
		'Dest': CategoricalDtype(categories=origin_options),
		'Distance': np.int64,
		'dep_delayed_15min': CategoricalDtype(categories=['N','Y']),
	}
)
data_test = pd.read_csv(
	path_test,
	dtype={
		'Month': CategoricalDtype(categories=month_options) ,
		'DayofMonth': CategoricalDtype(categories=day_of_month_options),
		'DayOfWeek': CategoricalDtype(categories=day_of_week_options),
		'DepTime': np.int64,
		'UniqueCarrier': CategoricalDtype(categories=carrier_options),
		'Origin': CategoricalDtype(categories=origin_options),
		'Dest': CategoricalDtype(categories=origin_options),
		'Distance': np.int64,
		'dep_delayed_15min': CategoricalDtype(categories=['N','Y']),
	}
)
target_column_name = "dep_delayed_15min"
data = pd.concat([data_train, data_test])
features = data.loc[:, data.columns != target_column_name]
labels = data[target_column_name]

if library == 'xgboost' or library == 'sklearn':
	features = pd.get_dummies(features)

(features_train, features_test, labels_train, labels_test) = train_test_split(
	features,
	labels,
	test_size=nrows_test,
	train_size=nrows_train,
	shuffle=False
)

# Train the model.
if library == 'h2o':
	import h2o
	from h2o.estimators import H2OGradientBoostingEstimator
	h2o.init()
	data_train = pd.concat([features_train, labels_train], axis=1)
	data_test = pd.concat([features_test, labels_test], axis=1)
	data_train = h2o.H2OFrame(python_obj=data_train)
	data_test = h2o.H2OFrame(python_obj=data_test)
	feature_column_names = [column for column in data_train.columns if column != target_column_name]
	model = H2OGradientBoostingEstimator(
		distribution="bernoulli",
		learn_rate = 0.1,
		max_depth = 10,
		nbins = 255,
		ntrees = 100,
	)
	model.train(
		training_frame=data_train,
		x=feature_column_names,
		y=target_column_name,
	)
elif library == 'lightgbm':
	import lightgbm as lgb
	model = lgb.LGBMClassifier(
		force_row_wise=True,
		learning_rate=0.1,
		n_estimators=100,
		num_leaves=255,
	)
	model.fit(
		features_train,
		labels_train,
	)
elif library == 'sklearn':
	from sklearn.experimental import enable_hist_gradient_boosting
	from sklearn.ensemble import HistGradientBoostingClassifier
	model = HistGradientBoostingClassifier(
		learning_rate=0.1,
		max_iter=100,
		max_leaf_nodes=512,
	)
	model.fit(features_train, labels_train)
elif library == 'xgboost':
	import xgboost as xgb
	model = xgb.XGBClassifier(
		eta = 0.1,
		grow_policy = 'lossguide',
		max_depth = 9,
		n_estimators = 100,
		tree_method = 'hist',
	)
	model.fit(features_train, labels_train)

# Compute metrics.
if library == 'h2o':
  predictions_proba = model.predict(data_test).as_data_frame()['True']
else:
  predictions_proba = model.predict_proba(features_test)[:, 1]
auc = roc_auc_score(labels_test, predictions_proba)
print('auc: ', auc)