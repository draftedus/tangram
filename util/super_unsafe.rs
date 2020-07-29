use std::cell::UnsafeCell;

pub struct SuperUnsafe<T>(UnsafeCell<T>);

unsafe impl<T> Sync for SuperUnsafe<T> {}

impl<T> SuperUnsafe<T> {
	pub fn new(value: T) -> Self {
		Self(UnsafeCell::new(value))
	}

	#[allow(clippy::mut_from_ref, clippy::missing_safety_doc)]
	pub unsafe fn get(&self) -> &mut T {
		&mut *self.0.get()
	}

	/// When you are done sharing your value super unsafely,
	/// you can return it back to safety by calling `.into_inner()`.
	pub fn into_inner(self) -> T {
		self.0.into_inner()
	}
}
