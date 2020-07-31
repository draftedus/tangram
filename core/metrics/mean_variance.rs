//! https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Parallel_algorithm

use num_traits::cast::ToPrimitive;

/// combine two separate means and variances into a single mean and variance
/// useful in parallel algorithms
pub fn merge_mean_m2(
	n_a: u64,
	mean_a: f64,
	m2_a: f64,
	n_b: u64,
	mean_b: f64,
	m2_b: f64,
) -> (f64, f64) {
	let n_a = n_a.to_f64().unwrap();
	let n_b = n_b.to_f64().unwrap();
	(
		(((n_a * mean_a) + (n_b * mean_b)) / (n_a + n_b)),
		m2_a + m2_b + (mean_b - mean_a) * (mean_b - mean_a) * (n_a * n_b / (n_a + n_b)),
	)
}

pub fn m2_to_variance(m2: f64, n: u64) -> f32 {
	(m2 / n.to_f64().unwrap()) as f32
}
