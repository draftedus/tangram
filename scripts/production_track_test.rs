use chrono::{TimeZone, Utc};
use clap::Clap;
use num_traits::ToPrimitive;
use rand::Rng;
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use tangram_app::common::monitor_event::{
	BinaryClassificationPredictOutput, MonitorEvent, MulticlassClassificationPredictOutput,
	NumberOrString, PredictOutput, PredictionMonitorEvent, RegressionPredictOutput,
	TrueValueMonitorEvent,
};
use tangram_dataframe::DataFrameView;
use tangram_util::error::Result;
use url::Url;

const NUM_EXAMPLES_TO_TRACK: usize = 1000;

#[derive(Clap)]
#[clap(
	about = "Track Predictions",
	setting = clap::AppSettings::DisableHelpSubcommand,
)]
struct Options {
	#[clap(name = "track", default_value = "http://localhost:8080/track")]
	_app_url: Url,
	#[clap(long)]
	model_name: String,
	#[clap(long)]
	model_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
	let options = Options::parse();
	let dataset = match options.model_name.as_str() {
		"heart_disease" => HEART_DISEASE,
		"mpg" => MPG,
		"iris" => IRIS,
		"boston" => BOSTON,
		_ => unimplemented!(),
	};
	let dataframe = tangram_dataframe::DataFrame::from_path(
		Path::new(dataset.csv_path),
		Default::default(),
		|_| {},
	)
	.unwrap();
	let mut rng = rand::thread_rng();
	for i in 0usize..NUM_EXAMPLES_TO_TRACK {
		let mut record = get_random_row(dataframe.view());
		let target = record.remove(dataset.target).unwrap();
		if dataset.name == "heart_disease" {
			// Rewrite asymptomatic to asx in 50% of rows.
			if rng.gen::<bool>() {
				let chest_pain = record.get_mut("chest_pain").unwrap();
				if chest_pain == "asymptomatic" {
					*chest_pain = serde_json::Value::String("asx".to_string());
				}
			}
		}
		let output = generate_fake_prediction(&target, &dataset);
		let model_id: &str = options.model_id.as_str();
		let date = get_random_date();
		let event: MonitorEvent = MonitorEvent::Prediction(PredictionMonitorEvent {
			date,
			identifier: NumberOrString::String(i.to_string()),
			input: record,
			output,
			model_id: model_id.parse().unwrap(),
		});
		track_event(event).await;
		if rng.gen::<f32>() > 0.4 {
			let event = MonitorEvent::TrueValue(TrueValueMonitorEvent {
				model_id: model_id.parse().unwrap(),
				identifier: NumberOrString::String(i.to_string()),
				true_value: target,
				date,
			});
			track_event(event).await;
		}
	}
	Ok(())
}

fn generate_fake_prediction(target: &serde_json::Value, dataset: &DatasetConfig) -> PredictOutput {
	match dataset.name {
		"heart_disease" => generate_fake_prediction_heart_disease(target, dataset),
		"mpg" => generate_fake_prediction_mpg(target),
		"iris" => generate_fake_prediction_iris(target, dataset),
		"boston" => generate_fake_prediction_boston(target),
		_ => unimplemented!(),
	}
}

fn generate_fake_prediction_heart_disease(
	target_value: &serde_json::Value,
	dataset: &DatasetConfig,
) -> PredictOutput {
	let mut rng = rand::thread_rng();
	let target_value = target_value.as_str().unwrap();
	let target_value = if rng.gen::<f32>() > 0.6 {
		target_value
	} else {
		let class_names = dataset.class_names.unwrap();
		let random_target_value_index = (rng.gen::<f32>() * class_names.len().to_f32().unwrap())
			.to_usize()
			.unwrap();
		class_names[random_target_value_index]
	};
	PredictOutput::BinaryClassification(BinaryClassificationPredictOutput {
		class_name: target_value.to_string(),
		probability: 0.95,
	})
}

fn generate_fake_prediction_mpg(target_value: &serde_json::Value) -> PredictOutput {
	let mut rng = rand::thread_rng();
	let target_value = target_value.as_f64().unwrap();
	let target_value = target_value + rng.gen::<f64>() * 5.0;
	PredictOutput::Regression(RegressionPredictOutput {
		value: target_value.to_f32().unwrap(),
	})
}

fn generate_fake_prediction_iris(
	target_value: &serde_json::Value,
	dataset: &DatasetConfig,
) -> PredictOutput {
	let mut rng = rand::thread_rng();
	let target_value = target_value.as_str().unwrap();
	let target_value = if rng.gen::<f32>() > 0.6 {
		target_value
	} else {
		let class_names = dataset.class_names.unwrap();
		let random_target_value_index = (rng.gen::<f32>() * class_names.len().to_f32().unwrap())
			.to_usize()
			.unwrap();
		class_names[random_target_value_index]
	};
	let probabilities = dataset
		.class_names
		.unwrap()
		.iter()
		.map(|class_name| {
			if class_name == &target_value {
				(class_name.to_string(), 0.95)
			} else {
				(class_name.to_string(), 0.025)
			}
		})
		.collect::<HashMap<String, f32>>();
	PredictOutput::MulticlassClassification(MulticlassClassificationPredictOutput {
		class_name: target_value.to_string(),
		probabilities: Some(probabilities),
	})
}

fn generate_fake_prediction_boston(target_value: &serde_json::Value) -> PredictOutput {
	let mut rng = rand::thread_rng();
	let target_value = target_value.as_f64().unwrap();
	let target_value = target_value + rng.gen::<f64>() * 5.0;
	PredictOutput::Regression(RegressionPredictOutput {
		value: target_value.to_f32().unwrap(),
	})
}

fn get_random_date() -> chrono::DateTime<Utc> {
	let start_time: u64 = 1577836800; // Jan 1 2020 00:00:00
	let mut rng = rand::thread_rng();
	let end_time = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_secs();
	let time_range = (end_time - start_time).to_f32().unwrap();
	let time: i64 =
		start_time.to_i64().unwrap() + (rng.gen::<f32>() * (time_range).trunc()).to_i64().unwrap();
	Utc.timestamp(time, 0)
}

fn get_random_row(dataframe: DataFrameView) -> HashMap<String, serde_json::Value> {
	let mut rng = rand::thread_rng();
	let random_row_index = (dataframe.nrows().to_f32().unwrap() * rng.gen::<f32>())
		.to_usize()
		.unwrap();
	dataframe
		.columns()
		.iter()
		.map(|column| match column {
			tangram_dataframe::DataFrameColumnView::Enum(column) => {
				let column_name = column.name().unwrap().to_owned();
				let value = column.data()[random_row_index];
				let value = match value {
					Some(value) => {
						serde_json::Value::String(column.options()[value.get() - 1].to_owned())
					}
					None => serde_json::Value::Null,
				};
				(column_name, value)
			}
			tangram_dataframe::DataFrameColumnView::Number(column) => {
				let column_name = column.name().unwrap().to_owned();
				let value = column.data()[random_row_index].to_f64().unwrap();
				let value = serde_json::Number::from_f64(value).unwrap();
				(column_name, serde_json::Value::Number(value))
			}
			_ => unimplemented!(),
		})
		.collect::<HashMap<String, serde_json::Value>>()
}

async fn track_event(event: MonitorEvent) {
	let client = reqwest::Client::new();
	let res = client
		.post("http://localhost:8080/track")
		.json(&event)
		.send()
		.await
		.unwrap();
	println!("{:?}", res);
}

struct DatasetConfig {
	csv_path: &'static str,
	name: &'static str,
	target: &'static str,
	class_names: Option<&'static [&'static str]>,
}

const HEART_DISEASE: DatasetConfig = DatasetConfig {
	csv_path: "data/heart_disease.csv",
	name: "heart_disease",
	target: "diagnosis",
	class_names: Some(&["Positive", "Negative"]),
};

const BOSTON: DatasetConfig = DatasetConfig {
	csv_path: "data/boston.csv",
	name: "boston",
	target: "medv",
	class_names: None,
};

const IRIS: DatasetConfig = DatasetConfig {
	csv_path: "data/iris.csv",
	name: "iris",
	target: "species",
	class_names: Some(&["Iris Setosa", "Iris Virginica", "Iris Versicolor"]),
};

const MPG: DatasetConfig = DatasetConfig {
	csv_path: "data/mpg.csv",
	name: "mpg",
	target: "mpg",
	class_names: None,
};
