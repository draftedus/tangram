use std::{borrow::Cow, collections::BTreeMap};
use tangram::{id::Id, util::finite::NotFiniteError};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Output {
	Regression(RegressionOutput),
	Classification(ClassificationOutput),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressionOutput {
	pub value: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationOutput {
	pub class_name: String,
	pub probabilities: Option<BTreeMap<String, f32>>,
}

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
	pub output: Output,
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
pub enum NumberOrString {
	Number(f32),
	String(String),
}

impl NumberOrString {
	pub fn as_number(&self) -> Result<f32, NotFiniteError> {
		match self {
			Self::Number(n) => Ok(*n),
			Self::String(s) => match lexical::parse::<f32, _>(s) {
				Ok(value) => Ok(value),
				Err(_) => Err(NotFiniteError),
			},
		}
	}
	pub fn as_string(&self) -> Cow<str> {
		match self {
			Self::Number(n) => n.to_string().into(),
			Self::String(s) => s.into(),
		}
	}
}
