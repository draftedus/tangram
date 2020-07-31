#![allow(clippy::all)]

use buffy::prelude::*;

use super::classifier::Classifier;
use super::regressor::Regressor;

#[derive(buffy::Serialize, buffy::Deserialize, Clone, Debug, PartialEq)]
pub enum Model {
	#[buffy(unknown)]
	UnknownVariant(u64, buffy::WireType, Vec<u8>),
	#[buffy(id = 1)]
	Regressor(Regressor),
	#[buffy(id = 2)]
	Classifier(Classifier),
}
