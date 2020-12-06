use std::{collections::HashMap, path::Path};

pub use include_out_dir_macro::include_out_dir;

pub struct Dir(pub HashMap<&'static Path, &'static [u8]>);

impl Dir {
	pub fn new(map: HashMap<&'static Path, &'static [u8]>) -> Dir {
		Dir(map)
	}

	pub fn read(&self, path: &Path) -> Option<&'static [u8]> {
		self.0.get(path).cloned()
	}
}
