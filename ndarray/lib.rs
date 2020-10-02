/*!
This crate provides a basic implementation of ndarrays, which are *n*-dimensional arrays of elements of the same data type. This crate is similar to Python's NumPy library, but at present is incredibly limited, because it only implements the features needed to support Tangram.
*/

// use num_traits::Num;

// pub struct Array<T, S>
// where
// 	S: Shape,
// {
// 	data: Vec<T>,
// 	shape: S,
// 	strides: S,
// }

// trait Shape {
// 	fn slice(&self) -> &[usize];

// 	fn size(&self) -> usize {
// 		self.slice().iter().fold(1, |a, b| a * b)
// 	}

// 	fn row_major_strides(&self) -> Self {}

// 	fn col_major_strides(&self) -> Self {}
// }

// impl Shape for usize {
// 	fn slice(&self) -> &[usize] {
// 		&[*self]
// 	}
// }

// impl Shape for (usize, usize) {
// 	fn slice(&self) -> &[usize] {
// 		&[self.0, self.1]
// 	}
// }

// impl<T, S> Array<T, S>
// where
// 	T: Num,
// 	S: Shape,
// {
// 	pub fn zeros(shape: S) -> Self {
// 		Self {
// 			data: vec![],
// 			shape,
// 			strides,
// 		}
// 	}
// }

// impl<T> std::ops::Index for Array<T> {

// }
