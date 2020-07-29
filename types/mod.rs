use crate::id::Id;
use anyhow::{format_err, Result};
use std::{
	io::{Read, Write},
	path::Path,
};

mod generated;

pub use self::generated::*;

impl Model {
	pub fn from_slice(slice: &[u8]) -> Result<Self> {
		let major_version = slice[0];
		if major_version != 0 {
			return Err(format_err!("unknown major version {}", major_version));
		}
		let slice = &slice[1..];
		let model: Self = buffy::from_slice(slice)?;
		Ok(model)
	}

	pub fn from_path(path: &Path) -> Result<Self> {
		let file = std::fs::File::open(path)?;
		let mut reader = std::io::BufReader::new(file);
		let mut major_version = [0u8; 1];
		reader.read_exact(&mut major_version)?;
		let major_version = major_version[0];
		if major_version != 0 {
			return Err(format_err!("unknown major version {}", major_version));
		}
		let model: Model = buffy::from_reader(&mut reader)?;
		Ok(model)
	}

	pub fn to_file(&self, path: &Path) -> Result<()> {
		let file = std::fs::File::create(path)?;
		let mut writer = std::io::BufWriter::new(file);
		writer.write_all(&[0])?;
		buffy::to_writer(&mut writer, self)?;
		Ok(())
	}

	pub fn id(&self) -> Id {
		match self {
			Self::Regressor(s) => s.id.as_option().unwrap().parse().unwrap(),
			Self::Classifier(s) => s.id.as_option().unwrap().parse().unwrap(),
			_ => unreachable!(),
		}
	}
}

impl Classifier {
	pub fn classes(&self) -> &[String] {
		match self.model.as_option().unwrap() {
			ClassificationModel::LinearBinary(model) => {
				model.classes.as_option().unwrap().as_slice()
			}
			ClassificationModel::GbtBinary(model) => model.classes.as_option().unwrap().as_slice(),
			ClassificationModel::LinearMulticlass(model) => {
				model.classes.as_option().unwrap().as_slice()
			}
			ClassificationModel::GbtMulticlass(model) => {
				model.classes.as_option().unwrap().as_slice()
			}
			_ => unreachable!(),
		}
	}
}

impl ColumnStats {
	pub fn column_name(&self) -> String {
		match &self {
			Self::Unknown(c) => c.column_name.as_option().unwrap().to_owned(),
			Self::Number(c) => c.column_name.as_option().unwrap().to_owned(),
			Self::Enum(c) => c.column_name.as_option().unwrap().to_owned(),
			Self::Text(c) => c.column_name.as_option().unwrap().to_owned(),
			Self::UnknownVariant(_, _, _) => unimplemented!(),
		}
	}

	pub fn as_number(&self) -> Option<&NumberColumnStats> {
		match self {
			Self::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<&EnumColumnStats> {
		match self {
			Self::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<&TextColumnStats> {
		match self {
			Self::Text(s) => Some(s),
			_ => None,
		}
	}
}
