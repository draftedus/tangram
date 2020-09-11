use num_traits::ToPrimitive;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Debug)]
pub struct Timing {
	pub binning: BinningTiming,
	pub bin_stats: BinStatsTiming,
	pub find_split: TimingDuration,
	pub rearrange_examples_index: TimingDuration,
	pub predict: TimingDuration,
	pub compute_feature_importances: TimingDuration,
	pub sum_gradients_hessians: TimingDuration,
	pub allocations: TimingDuration,
}

pub struct TimingDuration(AtomicU64);

#[derive(Debug)]
pub struct BinningTiming {
	pub compute_bin_info: TimingDuration,
	pub compute_binned_features: TimingDuration,
}

#[derive(Debug)]
pub struct BinStatsTiming {
	pub compute_bin_stats: TimingDuration,
	pub compute_bin_stats_subtraction: TimingDuration,
	pub compute_bin_stats_root: TimingDuration,
}

impl Timing {
	pub fn new() -> Self {
		Self {
			binning: BinningTiming {
				compute_bin_info: TimingDuration::new(),
				compute_binned_features: TimingDuration::new(),
			},
			bin_stats: BinStatsTiming {
				compute_bin_stats: TimingDuration::new(),
				compute_bin_stats_subtraction: TimingDuration::new(),
				compute_bin_stats_root: TimingDuration::new(),
			},
			sum_gradients_hessians: TimingDuration::new(),
			find_split: TimingDuration::new(),
			rearrange_examples_index: TimingDuration::new(),
			predict: TimingDuration::new(),
			compute_feature_importances: TimingDuration::new(),
			allocations: TimingDuration::new(),
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
