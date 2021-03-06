use std::{borrow::Cow, collections::HashMap};
use tangram_deps::{chrono, lexical, serde_json};
use tangram_util::finite::NotFiniteError;
use tangram_util::id::Id;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum MonitorEvent {
	#[serde(rename = "prediction")]
	Prediction(PredictionMonitorEvent),
	#[serde(rename = "true_value")]
	TrueValue(TrueValueMonitorEvent),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionMonitorEvent {
	pub model_id: Id,
	pub identifier: NumberOrString,
	pub input: HashMap<String, serde_json::Value>,
	pub output: PredictOutput,
	pub date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrueValueMonitorEvent {
	pub model_id: Id,
	pub identifier: NumberOrString,
	pub true_value: serde_json::Value,
	pub date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum PredictOutput {
	Regression(RegressionPredictOutput),
	BinaryClassification(BinaryClassificationPredictOutput),
	MulticlassClassification(MulticlassClassificationPredictOutput),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressionPredictOutput {
	pub value: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryClassificationPredictOutput {
	pub class_name: String,
	pub probability: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticlassClassificationPredictOutput {
	pub class_name: String,
	pub probabilities: Option<HashMap<String, f32>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum NumberOrString {
	Number(f32),
	String(String),
}

impl NumberOrString {
	pub fn as_number(&self) -> Result<f32, NotFiniteError> {
		match self {
			NumberOrString::Number(number) => Ok(*number),
			NumberOrString::String(string) => match lexical::parse::<f32, _>(string) {
				Ok(value) => Ok(value),
				Err(_) => Err(NotFiniteError),
			},
		}
	}
	pub fn as_string(&self) -> Cow<str> {
		match self {
			NumberOrString::Number(number) => number.to_string().into(),
			NumberOrString::String(string) => string.into(),
		}
	}
}
