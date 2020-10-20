use std::{borrow::Cow, collections::BTreeMap};
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
	pub input: BTreeMap<String, serde_json::Value>,
	pub output: PredictOutput,
	pub date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrueValueMonitorEvent {
	pub model_id: Id,
	pub identifier: NumberOrString,
	pub true_value: NumberOrString,
	pub date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum PredictOutput {
	Regression(RegressionOutput),
	MulticlassClassification(MulticlassClassificationOutput),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressionOutput {
	pub value: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticlassClassificationOutput {
	pub class_name: String,
	pub probabilities: Option<BTreeMap<String, f32>>,
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
			NumberOrString::Number(numbers) => Ok(*numbers),
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
