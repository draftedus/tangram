use anyhow::Result;
use chrono::{TimeZone, Utc};
use clap::Clap;
use num_traits::ToPrimitive;
use rand::Rng;
use reqwest::Response;
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use url::Url;

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
	track(TrackOptions {
		dataset,
		model_id: options.model_id,
	})
	.await;
	Ok(())
}

struct TrackOptions {
	dataset: DatasetConfig,
	model_id: String,
}

async fn track(options: TrackOptions) {
	let _dataframe = tangram_dataframe::DataFrame::from_path(
		Path::new(options.dataset.csv_path),
		Default::default(),
		|_| {},
	);
	let start_time: u64 = 1577836800; // Jan 1 2020 00:00:00
	let mut rng = rand::thread_rng();
	let end_time = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_secs();
	let time_range = (end_time - start_time).to_f32().unwrap();
	let time: i64 = (rng.gen::<f32>() * (time_range).trunc()).to_i64().unwrap();

	let date = Utc.timestamp(time, 0);
	for i in 0usize..1 {
		let event: Event = Event::Prediction(Prediction {
			date: date.to_rfc3339(),
			identifier: i.to_string(),
			input: json!({}),
			output: json!({}),
			model_id: options.model_id.to_string(),
		});
		track_event(event).await;
	}
}

async fn track_event(event: Event) {
	let client = reqwest::Client::new();
	let json_str = serde_json::to_string(&event).unwrap();
	let res = client
		.post("http://localhost:8080/track")
		.json(&json_str)
		.send()
		.await
		.unwrap();
	println!("{:?}", res);
}

#[derive(serde::Serialize)]
#[serde(tag = "type")]
enum Event {
	#[serde(rename = "prediction")]
	Prediction(Prediction),
	#[serde(rename = "true_value")]
	TrueValue(TrueValue),
}

#[derive(serde::Serialize)]
struct Prediction {
	date: String,
	identifier: String,
	input: serde_json::Value,
	model_id: String,
	output: serde_json::Value,
}

#[derive(serde::Serialize)]
struct TrueValue {
	date: String,
	identifier: String,
	model_id: String,
	true_value: String,
}

struct DatasetConfig {
	csv_path: &'static str,
	name: &'static str,
	target: &'static str,
	target_values: Option<&'static [&'static str]>,
}

const HEART_DISEASE: DatasetConfig = DatasetConfig {
	csv_path: "data/heart_disease.csv",
	name: "heart_disease",
	target: "diagnosis",
	target_values: Some(&["Positive", "Negative"]),
};
const BOSTON: DatasetConfig = DatasetConfig {
	csv_path: "data/boston.csv",
	name: "boston",
	target: "medv",
	target_values: None,
};
const IRIS: DatasetConfig = DatasetConfig {
	csv_path: "data/iris.csv",
	name: "iris",
	target: "species",
	target_values: Some(&["Iris Setosa", "Iris Virginica", "Iris Versicolor"]),
};
const MPG: DatasetConfig = DatasetConfig {
	csv_path: "data/mpg.csv",
	name: "mpg",
	target: "mpg",
	target_values: None,
};
