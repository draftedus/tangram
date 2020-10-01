/*!
This crate provides a basic implementation of ndarrays, which are *n*-dimensional arrays of homogeneous elements.
*/

pub struct Array<T> {
	pub data: Vec<T>,
}
