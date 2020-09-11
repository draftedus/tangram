use super::classifier::Classifier;
use super::regressor::Regressor;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum Model {
	Regressor(Regressor),
	Classifier(Classifier),
}
