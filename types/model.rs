use super::classifier::Classifier;
use super::regressor::Regressor;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Model {
	Regressor(Regressor),
	Classifier(Classifier),
}
