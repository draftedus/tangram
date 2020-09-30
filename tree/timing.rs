use num_traits::ToPrimitive;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Debug)]
pub struct Timing {
	pub allocations: TimingDuration,
	pub compute_bin_stats_root: TimingDuration,
	pub compute_bin_stats_subtraction: TimingDuration,
	pub compute_bin_stats: TimingDuration,
	pub compute_binned_features: TimingDuration,
	pub compute_binning_instructions: TimingDuration,
	pub compute_feature_importances: TimingDuration,
	pub compute_gradients_and_hessians: TimingDuration,
	pub find_split: TimingDuration,
	pub predict: TimingDuration,
	pub rearrange_examples_index: TimingDuration,
	pub sum_gradients_hessians: TimingDuration,
	pub train: TimingDuration,
}

pub struct TimingDuration(AtomicU64);

impl Timing {
	pub fn new() -> Self {
		Self {
			allocations: TimingDuration::new(),
			compute_bin_stats_root: TimingDuration::new(),
			compute_bin_stats_subtraction: TimingDuration::new(),
			compute_bin_stats: TimingDuration::new(),
			compute_binned_features: TimingDuration::new(),
			compute_binning_instructions: TimingDuration::new(),
			compute_feature_importances: TimingDuration::new(),
			compute_gradients_and_hessians: TimingDuration::new(),
			find_split: TimingDuration::new(),
			predict: TimingDuration::new(),
			rearrange_examples_index: TimingDuration::new(),
			sum_gradients_hessians: TimingDuration::new(),
			train: TimingDuration::new(),
		}
	}
}

impl TimingDuration {
	pub fn new() -> Self {
		Self(AtomicU64::new(0))
	}
	pub fn get(&self) -> Duration {
		Duration::from_nanos(self.0.load(Ordering::Relaxed))
	}
	pub fn inc(&self, value: Duration) -> u64 {
		self.0
			.fetch_add(value.as_nanos().to_u64().unwrap(), Ordering::Relaxed)
	}
}

impl std::fmt::Debug for TimingDuration {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.get())
	}
}
