use itertools::izip;
use maplit::btreemap;
use ndarray::prelude::*;
use serde_json::json;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use tangram_dataframe::prelude::*;
use tangram_metrics::{Metric, StreamingMetric};

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/flights_10m_train.csv");
	let csv_file_path_test = Path::new("data/flights_test.csv");
	let target_column_index = 8;
	let month_options = vec![
		"c-1", "c-10", "c-11", "c-12", "c-2", "c-3", "c-4", "c-5", "c-6", "c-7", "c-8", "c-9",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let day_of_week_options = vec!["c-1", "c-2", "c-3", "c-4", "c-5", "c-6", "c-7"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let day_of_month_options = vec![
		"c-1", "c-10", "c-11", "c-12", "c-13", "c-14", "c-15", "c-16", "c-17", "c-18", "c-19",
		"c-2", "c-20", "c-21", "c-22", "c-23", "c-24", "c-25", "c-26", "c-27", "c-28", "c-29",
		"c-3", "c-30", "c-31", "c-4", "c-5", "c-6", "c-7", "c-8", "c-9",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let carrier_options = vec![
		"AA", "AQ", "AS", "B6", "CO", "DH", "DL", "EV", "F9", "FL", "HA", "HP", "MQ", "NW", "OH",
		"OO", "TZ", "UA", "US", "WN", "XE", "YV",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let origin_options: Vec<String> = vec![
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
	.iter()
	.map(ToString::to_string)
	.collect();
	let dest_options: Vec<String> = vec![
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
	.iter()
	.map(ToString::to_string)
	.collect();
	let options = tangram_dataframe::FromCsvOptions {
		column_types: Some(btreemap! {
		  "month".to_owned() => DataFrameColumnType::Enum {options: month_options },
		  "day_of_week".to_owned() => DataFrameColumnType::Enum {options: day_of_week_options },
		  "day_of_month".to_owned() => DataFrameColumnType::Enum {options: day_of_month_options },
		  "dep_time".to_owned() => DataFrameColumnType::Number,
		  "unique_carrier".to_owned() => DataFrameColumnType::Enum { options: carrier_options },
		  "origin".to_owned() => DataFrameColumnType::Enum { options: origin_options },
		  "dest".to_owned() => DataFrameColumnType::Enum { options: dest_options },
		  "distance".to_owned() => DataFrameColumnType::Number,
		  "dep_delayed_15min".to_owned() => DataFrameColumnType::Enum { options: vec!["N".to_owned(), "Y".to_owned()] }
		}),
		..Default::default()
	};
	let mut features_train =
		DataFrame::from_path(csv_file_path_train, options.clone(), |_| {}).unwrap();
	let labels_train = features_train.columns_mut().remove(target_column_index);
	let labels_train = labels_train.as_enum().unwrap();
	let mut features_test =
		DataFrame::from_path(csv_file_path_test, options.clone(), |_| {}).unwrap();
	let labels_test = features_test.columns_mut().remove(target_column_index);
	let labels_test = labels_test.as_enum().unwrap();
	let feature_groups: Vec<tangram_features::FeatureGroup> = features_train
		.columns()
		.iter()
		.map(|column| {
			tangram_features::FeatureGroup::Normalized(
				tangram_features::NormalizedFeatureGroup::new(column.view()),
			)
		})
		.collect();
	let features_train = tangram_features::compute_features_array_f32(
		&features_train.view(),
		feature_groups.as_slice(),
		&|| {},
	);
	let features_test = tangram_features::compute_features_array_f32(
		&features_test.view(),
		feature_groups.as_slice(),
		&|| {},
	);

	// Train the model.
	let train_output = tangram_linear::BinaryClassifier::train(
		features_train.view(),
		labels_train.view(),
		&tangram_linear::TrainOptions {
			learning_rate: 0.01,
			max_epochs: 1,
			n_examples_per_batch: 1000,
			..Default::default()
		},
		&mut |_| {},
	);

	// Make predictions on the test data.
	let mut probabilities = Array::zeros(labels_test.len());
	train_output
		.model
		.predict(features_test.view(), probabilities.view_mut());

	// Compute metrics.
	let mut metrics = tangram_metrics::BinaryClassificationMetrics::new(3);
	metrics.update(tangram_metrics::BinaryClassificationMetricsInput {
		probabilities: probabilities.view().as_slice().unwrap(),
		labels: labels_test.view().data(),
	});
	let metrics = metrics.finalize();
	let input = izip!(probabilities.iter(), labels_test.iter())
		.map(|(probability, label)| (*probability, label.unwrap()))
		.collect();
	let auc_roc = tangram_metrics::AUCROC::compute(input);

	// Compute memory usage.
	let mut memory = String::new();
	let file = File::open("/proc/self/status").unwrap();
	for line in BufReader::new(file).lines().map(|l| l.unwrap()) {
		if line.starts_with("VmHWM") {
			memory = line.split(':').nth(1).map(|x| x.trim().to_owned()).unwrap();
		}
	}

	let output = json!({
		"auc_roc": auc_roc,
		"accuracy": metrics.thresholds[metrics.thresholds.len() / 2].accuracy,
		"memory": memory
	});
	println!("{}", output);
}
