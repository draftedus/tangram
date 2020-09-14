/*!
`SuperUnsafe` is the ultimate escape hatch to the Rust borrow checker. With it, you can simultaneously hold multiple mutable references to the same value, allowing you to process the value concurrently from multiple threads.
*/

use std::cell::UnsafeCell;

pub struct SuperUnsafe<T>(UnsafeCell<T>);

unsafe impl<T> Sync for SuperUnsafe<T> {}

impl<T> SuperUnsafe<T> {
	/// Wrap a value with SuperUnsafe in preparation to acquire multiple mutable references to it.
	pub fn new(value: T) -> Self {
		Self(UnsafeCell::new(value))
	}

	/// Get a mutable reference to your value with absolutely no borrow checking. Make sure you know what you are doing!
	#[allow(clippy::mut_from_ref, clippy::missing_safety_doc)]
	pub unsafe fn get(&self) -> &mut T {
		&mut *self.0.get()
	}

	/// When you are done, you can return your value back to safety by calling `.into_inner()`.
	pub fn into_inner(self) -> T {
		self.0.into_inner()
	}
}
