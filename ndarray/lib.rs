/*!
This crate provides a basic implementation of ndarrays, which are *n*-dimensional arrays of elements of the same data type. This crate is similar to Python's NumPy library, but at present is incredibly limited, because it only implements the features needed to support Tangram.
*/

use itertools::izip;

pub struct Array<T, D>
where
	D: Dimension,
{
	data: Vec<T>,
	shape: D,
	strides: D,
}

pub trait Dimension: Sized {
	fn slice(&self) -> &[usize];

	fn size(&self) -> usize {
		self.slice().iter().product()
	}

	fn row_major_strides(&self) -> Self {
		todo!()
	}

	fn col_major_strides(&self) -> Self {
		todo!()
	}
}

fn offset_checked<D>(shape: &D, strides: &D, index: &D) -> Option<usize>
where
	D: Dimension,
{
	let mut offset = 0;
	for (shape, stride, index) in izip!(shape.slice(), strides.slice(), index.slice()) {
		if index >= shape {
			return None;
		}
		offset += stride * index;
	}
	Some(offset)
}

struct Dim<S>(S);

impl Dimension for Dim<[usize; 1]> {
	fn slice(&self) -> &[usize] {
		&self.0
	}
}

impl Dimension for Dim<[usize; 2]> {
	fn slice(&self) -> &[usize] {
		&self.0
	}
}

// impl<T, S> Array<T, S>
// where
// 	T: Num,
// 	S: Dimension,
// {
// 	pub fn zeros(shape: S) -> Self {
// 		Self {
// 			data: vec![],
// 			shape,
// 			strides: shape.row_major_strides(),
// 		}
// 	}
// }

impl<T, D> std::ops::Index<D> for Array<T, D>
where
	D: Dimension,
{
	type Output = T;
	fn index(&self, index: D) -> &Self::Output {
		let offset = offset_checked(&self.shape, &self.strides, &index).unwrap();
		unsafe { self.data.get_unchecked(offset) }
	}
}
