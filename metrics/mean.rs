use super::StreamingMetric;
use num_traits::ToPrimitive;
use std::num::NonZeroU64;

/// The Mean metric is computed using [Welford's algorithm](https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Parallel_algorithm) to ensure numeric stability.
#[derive(Debug, Clone, Default)]
pub struct Mean(Option<(NonZeroU64, f64)>);

impl Mean {
	pub fn new() -> Self {
		Self::default()
	}
}

impl StreamingMetric<'_> for Mean {
	type Input = f32;
	type Output = Option<f32>;

	fn update(&mut self, value: f32) {
		let value = value.to_f64().unwrap();
		let one = NonZeroU64::new(1u64).unwrap();
		self.0 = match self.0 {
			None => Some((one, value)),
			Some(n_mean) => {
				let (n, mean) = n_mean;
				let new_mean = merge(mean, n, value, one);
				Some((NonZeroU64::new(n.get() + 1).unwrap(), new_mean))
			}
		}
	}

	fn merge(&mut self, other: Self) {
		self.0 = match (self.0, other.0) {
			(None, None) => None,
			(None, Some((n, mean))) => Some((n, mean)),
			(Some((n, mean)), None) => Some((n, mean)),
			(Some((n_a, mean_a)), Some((n_b, mean_b))) => Some((
				NonZeroU64::new(n_a.get() + n_b.get()).unwrap(),
				merge(mean_a, n_a, mean_b, n_b),
			)),
		};
	}

	fn finalize(self) -> Option<f32> {
		self.0.map(|(_, mean)| mean.to_f32().unwrap())
	}
}

/// This function merges two means together given their means and n's.
fn merge(mean_a: f64, n_a: NonZeroU64, mean_b: f64, n_b: NonZeroU64) -> f64 {
	let n_a = n_a.get().to_f64().unwrap();
	let n_b = n_b.get().to_f64().unwrap();
	((n_a * mean_a) + (n_b * mean_b)) / (n_a + n_b)
}
