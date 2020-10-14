use maplit::btreemap;
use ndarray::prelude::*;
use rayon::prelude::*;
use std::path::Path;
use tangram_dataframe::prelude::*;
use tangram_metrics::Metric;
use tangram_util::pzip;

fn main() {
	let start = std::time::Instant::now();
	// Load the data.
	// let csv_file_path_train = Path::new("data/flights-100k.csv");
	// let csv_file_path_test = Path::new("data/flights-test.csv");
	// let csv_file_path_train = Path::new("data/flights-1m.csv");
	// let csv_file_path_test = Path::new("data/flights-test.csv");
	let csv_file_path_train = Path::new("data/flights-10m.csv");
	let csv_file_path_test = Path::new("data/flights-test.csv");
	let target_column_index = 8;
	let month_options = vec![
		"c-1", "c-10", "c-11", "c-12", "c-2", "c-3", "c-4", "c-5", "c-6", "c-7", "c-8", "c-9",
	]
	.iter()
	.map(|m| m.to_string())
	.collect();
	let day_of_week_options = vec!["c-1", "c-2", "c-3", "c-4", "c-5", "c-6", "c-7"]
		.iter()
		.map(|m| m.to_string())
		.collect();
	let day_of_month_options = vec![
		"c-1", "c-10", "c-11", "c-12", "c-13", "c-14", "c-15", "c-16", "c-17", "c-18", "c-19",
		"c-2", "c-20", "c-21", "c-22", "c-23", "c-24", "c-25", "c-26", "c-27", "c-28", "c-29",
		"c-3", "c-30", "c-31", "c-4", "c-5", "c-6", "c-7", "c-8", "c-9",
	]
	.iter()
	.map(|m| m.to_string())
	.collect();
	let carrier_options = vec![
		"AA", "AQ", "AS", "B6", "CO", "DH", "DL", "EV", "F9", "FL", "HA", "HP", "MQ", "NW", "OH",
		"OO", "TZ", "UA", "US", "WN", "XE", "YV",
	]
	.iter()
	.map(|m| m.to_string())
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
	.map(|m| m.to_string())
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
	.map(|m| m.to_string())
	.collect();
	let options = tangram_dataframe::FromCsvOptions {
		column_types: Some(btreemap! {
		  "Month".into() => DataFrameColumnType::Enum {options: month_options },
		  "DayOfWeek".into() => DataFrameColumnType::Enum {options: day_of_week_options },
		  "DayofMonth".into() => DataFrameColumnType::Enum {options: day_of_month_options },
		  "DepTime".into() => DataFrameColumnType::Number,
		  "UniqueCarrier".into() => DataFrameColumnType::Enum { options: carrier_options },
		  "Origin".into() => DataFrameColumnType::Enum { options: origin_options },
		  "Dest".into() => DataFrameColumnType::Enum { options: dest_options },
		  "Distance".into() => DataFrameColumnType::Number,
		  "dep_delayed_15min".into() => DataFrameColumnType::Enum { options: vec!["N".into(), "Y".into()] }
		}),
		..Default::default()
	};
	let mut features_train =
		DataFrame::from_path(csv_file_path_train, options.clone(), |_| {}).unwrap();
	let labels_train = features_train.columns.remove(target_column_index);
	let labels_train = labels_train.as_enum().unwrap();
	let mut features_test = DataFrame::from_path(csv_file_path_test, options, |_| {}).unwrap();
	let labels_test = features_test.columns.remove(target_column_index);
	let labels_test = labels_test.as_enum().unwrap();
	let load_duration = start.elapsed();

	// Train the model.
	let start = std::time::Instant::now();
	let train_options = tangram_tree::TrainOptions {
		binned_features_layout: tangram_tree::BinnedFeaturesLayout::RowMajor,
		learning_rate: 0.1,
		max_rounds: 100,
		max_leaf_nodes: 255,
		..Default::default()
	};
	let model = tangram_tree::BinaryClassifier::train(
		features_train.view().clone(),
		labels_train.view(),
		&train_options,
		&mut |_| {},
	);
	let duration = start.elapsed();

	// Make predictions on the test data.
	let features_test = features_test.to_rows();
	let chunk_size =
		(features_test.nrows() + rayon::current_num_threads() - 1) / rayon::current_num_threads();
	let mut probabilities: Array2<f32> =
		unsafe { Array2::uninitialized((features_test.nrows(), 2)) };
	pzip!(
		features_test.axis_chunks_iter(Axis(0), chunk_size),
		probabilities.axis_chunks_iter_mut(Axis(0), chunk_size),
	)
	.for_each(|(features_test_chunk, probabilities_chunk)| {
		model.predict(features_test_chunk, probabilities_chunk);
	});

	// Compute metrics.
	let start = std::time::Instant::now();
	let labels = labels_test;
	let input = probabilities
		.column(1)
		.iter()
		.cloned()
		.zip(labels.data.iter().map(|d| d.unwrap()))
		.collect();
	let auc_roc = tangram_metrics::AUCROC::compute(input);
	let metrics_duration = start.elapsed();
	println!("load duration {:?}", load_duration);
	println!("train duration {:?}", duration);
	println!("metrics duration: {:?}", metrics_duration);
	println!("auc: {}", auc_roc);
}
