use super::BinDirections;

/// Categorical splits are represented as a space-efficient bit vector.
/// If the entry at index i is 0, then the i-th enum variant goes to the left subtree
/// and if the value is 1, the i-th enum variant goes to the right subtree.
impl BinDirections {
	pub fn new(n: u8, value: bool) -> Self {
		let bytes = if !value { [0x00; 32] } else { [0xFF; 32] };
		Self { n, bytes }
	}

	/// Retrieves the bin direction for the enum variant given by `index`.
	/// Returns `None` if the index is greater than the total number of enum variants (n).
	pub fn get(&self, index: u8) -> Option<bool> {
		if index >= self.n {
			None
		} else {
			let byte_index = (index / 8) as usize;
			let byte = self.bytes[byte_index];
			let bit_index = index % 8;
			let bit = (byte >> bit_index) & 0b0000_0001;
			Some(bit == 1)
		}
	}

	/// Sets the bin direction for the given enum variant at `index` to the value passed, 0 if this enum variant should go the the left subtree and 1 if it should go to the right.
	pub fn set(&mut self, index: u8, value: bool) {
		let byte_index = (index / 8) as usize;
		let bit_index = index % 8;
		if value {
			self.bytes[byte_index] |= 1 << bit_index;
		} else {
			self.bytes[byte_index] &= !(1 << bit_index);
		}
	}
}
