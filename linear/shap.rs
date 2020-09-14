use itertools::izip;
use ndarray::prelude::*;

/// compute the shap values for a single example
pub fn compute_shap(
	example: ArrayView1<f32>,
	bias: f32,
	weights: ArrayView1<f32>,
	means: &[f32],
	shap_values: &mut [f32],
) {
	let mut bias_shap_value: f32 = bias;
	bias_shap_value += weights
		.iter()
		.zip(means.iter())
		.map(|(weight, mean)| weight * mean)
		.sum::<f32>();
	let len = shap_values.len();
	shap_values[len - 1] = bias_shap_value;
	for (shap_value, weight, feature, mean) in
		izip!(&mut shap_values[0..len - 1], weights, example, means)
	{
		*shap_value = weight * (feature - mean);
	}
}
