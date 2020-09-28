#![allow(clippy::tabs_in_doc_comments)]

use std::sync::{
	atomic::{AtomicU64, Ordering},
	Arc,
};

/**
A `ProgressCounter` is used to efficiently track the progress of a task occurring across multiple threads.

Imagine you have the following code to ship order in parallel:

```ignore
orders.par_iter_mut().for_each(|order| { order.ship() });
```

Now you want to track the progress of this loop. You can use `Arc<Mutex<T>>` like so:

```ignore
use std::sync::{Arc, Mutex};

let progress_counter = Arc::new(Mutex::new(0));
orders.par_iter_mut().for_each(|order| {
	order.ship();
	*progress_counter.lock().unwrap() += 1;
});
```

However, if `ship_order` is sufficiently fast, a large portion of each thread's time will be spent waiting on the mutex. A better choice in this case is to use [atomics](https://doc.rust-lang.org/stable/std/sync/atomic/index.html). `ProgressCounter` is a convenient wrapper around atomics for use in tracking progress. This example will now run much faster:

```ignore
use tangram_progress::ProgressCounter;

let progress_counter = ProgressCounter::new(orders.len() as u64);
orders.par_iter_mut().for_each(|i| {
	ship_order(i);
	progress_counter.inc();
});
```
*/
#[derive(Clone, Debug)]
pub struct ProgressCounter {
	current: Arc<AtomicU64>,
	total: u64,
}

impl ProgressCounter {
	/// Create a new `ProgressCounter` that will count from 0 up to the specified `total`.
	pub fn new(total: u64) -> Self {
		Self {
			current: Arc::new(AtomicU64::new(0)),
			total,
		}
	}
	/// Retrieve the total value this `ProgressCounter` counts up to.
	pub fn total(&self) -> u64 {
		self.total
	}
	/// Retrieve the current progress value.
	pub fn get(&self) -> u64 {
		self.current.load(Ordering::Relaxed)
	}
	/// Set the current progress value.
	pub fn set(&self, value: u64) {
		self.current.store(value, Ordering::Relaxed);
	}
	/// Increment the progress value by `amount`.
	pub fn inc(&self, amount: u64) {
		self.current.fetch_add(amount, Ordering::Relaxed);
	}
}
