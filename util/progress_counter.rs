use std::sync::{
	atomic::{AtomicU64, Ordering},
	Arc,
};

#[derive(Clone, Debug)]
pub struct ProgressCounter {
	current: Arc<AtomicU64>,
	total: u64,
}

impl ProgressCounter {
	pub fn new(total: u64) -> Self {
		Self {
			current: Arc::new(AtomicU64::new(0)),
			total,
		}
	}
	pub fn total(&self) -> u64 {
		self.total
	}
	pub fn get(&self) -> u64 {
		self.current.load(Ordering::Relaxed)
	}
	pub fn set(&self, value: u64) {
		self.current.store(value, Ordering::Relaxed);
	}
	pub fn inc(&self, amount: u64) {
		self.current.fetch_add(amount, Ordering::Relaxed);
	}
}
