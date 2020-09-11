struct Array<T> {
	data: Vec<T>,
	shape: S,
}

struct ArrayView<T, D> {
	data: &[T],
	shape: D,
	strides: D,
}
