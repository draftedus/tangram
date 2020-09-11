mod classifier;
mod features;
mod regressor;
mod stats;
mod train_options;
mod tree;

pub use classifier::*;
pub use features::*;
pub use regressor::*;
pub use stats::*;
pub use train_options::*;
pub use tree::*;

use crate::id::Id;
use anyhow::{format_err, Result};
use std::{
	io::{Read, Write},
	path::Path,
};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Model {
	Regressor(Regressor),
	Classifier(Classifier),
}

impl Model {
	/// Deserialize a `Model` from a slice.
	pub fn from_slice(slice: &[u8]) -> Result<Self> {
		let major_version = slice[0];
		if major_version != 0 {
			return Err(format_err!("unknown major version {}", major_version));
		}
		let slice = &slice[1..];
		let model: Self = rmp_serde::from_slice(slice)?;
		Ok(model)
	}

	/// Deserialize a `Model` by reading the file at `path`.
	pub fn from_path(path: &Path) -> Result<Self> {
		let file = std::fs::File::open(path)?;
		let mut reader = std::io::BufReader::new(file);
		let mut major_version = [0u8; 1];
		reader.read_exact(&mut major_version)?;
		let major_version = major_version[0];
		if major_version != 0 {
			return Err(format_err!("unknown major version {}", major_version));
		}
		let model: Model = rmp_serde::from_read(&mut reader)?;
		Ok(model)
	}

	/// Write this model to the file at `path`.
	pub fn to_file(&self, path: &Path) -> Result<()> {
		let file = std::fs::File::create(path)?;
		let mut writer = std::io::BufWriter::new(file);
		writer.write_all(&[0])?;
		rmp_serde::encode::write_named(&mut writer, self)?;
		Ok(())
	}

	/// Retrieve this `Model`'s `Id`.
	pub fn id(&self) -> Id {
		match self {
			Self::Regressor(s) => s.id.parse().unwrap(),
			Self::Classifier(s) => s.id.parse().unwrap(),
		}
	}
}
