#include <stdlib.h>
#include <stdio.h>

void compute_bin_stats_for_feature_not_root(
		size_t n_examples,
		float* ordered_gradients,
		float* ordered_hessians,
		u_int8_t* binned_feature_values,
		double* bin_stats_for_feature,
		size_t* examples_index
) {
	double *bin_stats_gradients = bin_stats_for_feature;
	double *bin_stats_hessians = bin_stats_for_feature + 1;
	for (size_t i = 0; i < n_examples; i++) {
		__builtin_prefetch((void*)examples_index[i + 64], 0, 3);
		u_int8_t binned_feature_value = binned_feature_values[examples_index[i]];
		size_t bin_index = binned_feature_value << 1;
		bin_stats_gradients[bin_index] += ordered_gradients[i];
		bin_stats_hessians[bin_index] += ordered_hessians[i];
	}
}
