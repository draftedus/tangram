use anyhow::Result;
use maplit::btreemap;
use ndarray::prelude::*;
use std::path::Path;
use std::time::Instant;
use tangram::dataframe::*;
use tangram::metrics;

fn main() -> Result<()> {
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
		"SGF", "SGU", "SHV", "SIT", "SJC", "SJT", "SJU",
	]
	.iter()
	.map(|m| m.to_string())
	.collect();
	/*"SLC", "SMF", "SMX","SNA", "SOP", "SPI", "SPS", "SRQ", "STL", "STT", "STX", "SUN", "SWF", "SYR", "TEX", "TLH", "TOL", "TPA", "TRI",
		"TTN", "TUL", "TUP", "TUS", "TVC", "TWF", "TXK", "TYR", "TYS", "VCT", "VIS", "VLD", "VPS",
		"WRG", "WYS", "XNA", "YAK", "YUM",
	]
	.iter()
	.map(|m| m.to_string())
	.collect();
		*/
	let _dest_options: Vec<String> = vec![
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
	let csv_file_path_train = Path::new("data/flights-10m.csv");
	let csv_file_path_test = Path::new("data/flights-test.csv");
	let _nrows_train = 10_000_000;
	let _nrows_test = 100_000;
	let target_column_index = 8;
	let options = FromCsvOptions {
		column_types: Some(btreemap! {
		  "Month".into() => ColumnType::Enum {options: month_options},
		  "DayOfWeek".into() => ColumnType::Enum {options: day_of_week_options},
		  "DayOfMonth".into() => ColumnType::Enum {options: day_of_month_options},
		  "DepTime".into() => ColumnType::Number,
		  "UniqueCarrier".into() => ColumnType::Enum { options: carrier_options},
		  "Origin".into() => ColumnType::Enum { options: origin_options.clone()},
		  "Dest".into() => ColumnType::Enum { options: origin_options},
		  "Distance".into() => ColumnType::Number,
		  "dep_delayed_15min".into() => ColumnType::Enum { options: vec!["N".into(), "Y".into()]}
		}),
		infer_options: InferOptions {
			enum_max_unique_values: 292,
		},
	};
	let mut csv_reader = csv::Reader::from_path(csv_file_path_train)?;
	let mut dataframe_train = DataFrame::from_csv(&mut csv_reader, options.clone(), |_| {})?;
	let labels_train = dataframe_train.columns.remove(target_column_index);
	let labels_train = labels_train.as_enum().unwrap();

	let mut csv_reader = csv::Reader::from_path(csv_file_path_test)?;
	let mut dataframe_test = DataFrame::from_csv(&mut csv_reader, options, |_| {})?;
	let labels_test = dataframe_test.columns.remove(target_column_index);
	let labels_test = labels_test.as_enum().unwrap();

	// compute stats
	let stats_settings = tangram::stats::StatsSettings {
		number_histogram_max_size: 100,
		text_histogram_max_size: 100,
	};
	// retrieve the column names
	let column_names: Vec<String> = dataframe_train
		.columns
		.iter()
		.map(|column| column.name().to_owned())
		.collect();

	let tangram::stats::ComputeStatsOutput {
		overall_column_stats,
		..
	} = tangram::stats::compute_stats(
		&column_names,
		&dataframe_train.view(),
		&dataframe_test.view(),
		&stats_settings,
		&mut |_| {},
	);
	let feature_groups = tangram::features::compute_feature_groups_tree(&overall_column_stats);
	let features_train = tangram::features::compute_features_dataframe(
		&dataframe_train.view(),
		&feature_groups,
		&|| {},
	);

	// train the model
	let train_options = tangram::tree::TrainOptions {
		learning_rate: 0.1,
		max_rounds: 100,
		max_leaf_nodes: 512,
		max_depth: 10,
		..Default::default()
	};

	let start = Instant::now();
	let model = tangram::tree::BinaryClassifier::train(
		features_train.view().clone(),
		labels_train.view(),
		train_options,
		&mut |_| {},
	);
	let end = Instant::now();
	println!("duration: {:?}", end - start);

	let n_features = dataframe_train.ncols();
	let mut features_test = unsafe { Array2::uninitialized((dataframe_test.nrows(), n_features)) };
	tangram::features::compute_features_ndarray_value(
		&dataframe_test.view(),
		&feature_groups,
		features_test.view_mut(),
		&|| {},
	);

	let mut probabilities: Array2<f32> =
		unsafe { Array::uninitialized((features_test.nrows(), 2).f()) };
	model.predict(features_test.view(), probabilities.view_mut(), None);
	let accuracy = metrics::accuracy(probabilities.view(), labels_test.view().data.into());
	let baseline_accuracy = labels_test
		.data
		.iter()
		.map(|label| label.checked_sub(1).unwrap())
		.sum::<usize>() as f32
		/ labels_test.data.len() as f32;
	println!("baseline accuracy: {:?}", baseline_accuracy);
	println!("accuracy: {:?}", accuracy);
	let auc = metrics::auc_roc(
		&probabilities.column(1).as_slice().unwrap(),
		&labels_test.view().data,
	);
	println!("auc: {:?}", auc);
	Ok(())
}
