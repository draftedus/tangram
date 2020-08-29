use super::classifier::Classifier;
use super::regressor::Regressor;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum Model {
	Regressor(Regressor),
	Classifier(Classifier),
}
