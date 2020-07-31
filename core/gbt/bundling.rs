// use itertools::Itertools;
// use ndarray::prelude::*;
// use ndarray::Zip;
// use num_traits::ToPrimitive;

// const MAX_BINS_FOR_BUNDLE: usize = 255;

// /// returns an Array1 where value of the i-th entry contains information about the feature bundle that the original feature was mapped into
// /// each feature will be mapped to a feature bundle
// /// each feature bundle will contain 1 or more features
// pub fn compute_feature_bundles(
// 	// (n_examples, n_features)
// 	binned_features: ArrayView2<u8>,
// 	binning_thresholds: &[Vec<f32>],
// ) -> (
// 	Array1<FeatureBundle>,
// 	Array1<FeatureToFeatureBundle>,
// 	Vec<Vec<f32>>,
// ) {
// 	// sort the features by the number of nonzero values they have
// 	let nonzero_bin_counts: Vec<usize> = binned_features
// 		.gencolumns()
// 		.into_iter()
// 		.map(|binned_feature| {
// 			binned_feature.iter().fold(0, |mut acc, bin| {
// 				if *bin != 0 {
// 					acc += 1;
// 				}
// 				acc
// 			})
// 		})
// 		.collect();

// 	// sort by nonzero_bin_counts
// 	let mut sorted_feature_indexes = nonzero_bin_counts
// 		.into_iter()
// 		.enumerate()
// 		.collect::<Vec<(usize, usize)>>();
// 	sorted_feature_indexes.sort_by(|a, b| b.1.cmp(&a.1));

// 	let mut feature_bundles: Vec<FeatureBundle> = Vec::new();

// 	// we need to create a new group for this feature
// 	for (feature_index, _) in sorted_feature_indexes {
// 		let mut create_new_group: bool = true;
// 		let iter = feature_bundles.iter_mut().zip(binning_thresholds.iter());
// 		for (feature_bundle, binning_thresholds) in iter {
// 			// nonzero_bins_current_feature_group[feature_group_index]
// 			// if this feature_bundle is already full skip it
// 			let n_bins_for_feature = binning_thresholds.len() + 1;
// 			if feature_bundle.n_bins.to_usize().unwrap() + n_bins_for_feature >= MAX_BINS_FOR_BUNDLE
// 			{
// 				continue;
// 			}
// 			let overlap_count = compute_nonzero_bins_overlap_count(
// 				feature_bundle.has_nonzero_bin_value.view(),
// 				binned_features.column(feature_index),
// 			);
// 			// compute how many bins overlap between these two groups
// 			if overlap_count == 0 {
// 				// add this feature to the feature group
// 				feature_bundle.add_feature(feature_index, binned_features.column(feature_index));
// 				create_new_group = false;
// 				break;
// 			}
// 		}
// 		if create_new_group {
// 			let feature_bundle =
// 				FeatureBundle::new(feature_index, binned_features.column(feature_index));
// 			feature_bundles.push(feature_bundle);
// 		}
// 	}

// 	let n_features = binned_features.ncols();
// 	let mut feature_to_feature_bundle = unsafe { Array1::uninitialized(n_features) };

// 	let bundle_binning_thresholds = feature_bundles
// 		.iter()
// 		.enumerate()
// 		.map(|(feature_bundle_index, feature_bundle)| {
// 			let binning_thresholds = if feature_bundle.features.len() == 1 {
// 				// binning thresholds same as parent feature
// 				binning_thresholds[feature_bundle.features[0].feature_index].clone()
// 			} else {
// 				// categorical binning thresholds
// 				(0..feature_bundle.n_bins.to_usize().unwrap())
// 					.tuple_windows()
// 					.map(|(a, b)| (a.to_f32().unwrap() + b.to_f32().unwrap()) / 2.0)
// 					.collect()
// 			};
// 			feature_bundle.features.iter().for_each(|feature| {
// 				let feature_index = feature.feature_index;
// 				feature_to_feature_bundle[feature_index] = FeatureToFeatureBundle {
// 					offset: feature.offset,
// 					feature_bundle_index,
// 				}
// 			});
// 			binning_thresholds
// 		})
// 		.collect();

// 	(
// 		feature_bundles.into(),
// 		feature_to_feature_bundle,
// 		bundle_binning_thresholds,
// 	)
// }

// /// output  (n_examples, n_feature_groups)
// // reduces the training dataset to n_examples x n_feature_groups
// pub fn compute_binned_feature_bundles(
// 	// (n_examples, n_features)
// 	binned_features: Array2<u8>,
// 	// (n_features)
// 	feature_bundles: ArrayView1<FeatureBundle>,
// ) -> Array2<u8> {
// 	// modify the values in the feature groups to create a single feature group ready to pass to the train function
// 	// https://github.com/microsoft/LightGBM/blob/25d149d8ceb92838dbf2f7331f9dc0dec701ffd8/src/io/dataset.cpp#L92
// 	let n_feature_bundles = feature_bundles.len();
// 	let n_examples = binned_features.nrows();

// 	let mut binned_feature_bundles: Array2<u8> =
// 		unsafe { Array2::uninitialized((n_examples, n_feature_bundles)) };

// 	Zip::from(binned_feature_bundles.gencolumns_mut())
// 		.and(feature_bundles.axis_iter(Axis(0)))
// 		.apply(|mut binned_feature_bundle, feature_bundle| {
// 			// map the features in a group to a binned_feature_group
// 			// also generate the feature_to_feature_groups_lookup
// 			let feature_bundle = feature_bundle.into_scalar();
// 			for feature in feature_bundle.features.iter() {
// 				binned_feature_bundle
// 					.iter_mut()
// 					.zip(binned_features.column(feature.feature_index))
// 					.for_each(|(binned_group, binned_feature)| {
// 						if *binned_feature != 0 {
// 							*binned_group += binned_feature + feature.offset;
// 						}
// 					});
// 			}
// 		});

// 	binned_feature_bundles
// }

// fn compute_nonzero_bins_overlap_count(
// 	used_bins_for_feature_group: ArrayView1<bool>,
// 	binned_features: ArrayView1<u8>,
// ) -> usize {
// 	// if the feature u8 is nonzero, and the feature group value is true then there is overlap between the nonzero bins
// 	used_bins_for_feature_group
// 		.iter()
// 		.zip(binned_features)
// 		.fold(0, |mut acc, (is_used, binned_feature)| {
// 			if *is_used && *binned_feature != 0 {
// 				acc += 1;
// 			}
// 			acc
// 		})
// }

// #[derive(Copy, Clone, Debug)]
// pub struct FeatureToFeatureBundle {
// 	/// index of the feature group
// 	pub feature_bundle_index: usize,
// 	/// value to add to the original feature to get the value of the feature in the feature group
// 	pub offset: u8,
// }
// #[derive(Debug)]
// pub struct FeatureBundle {
// 	// for each example index indicates whether any feature in the bundle has a nonzero bin value
// 	pub has_nonzero_bin_value: Array1<bool>,
// 	pub features: Vec<Feature>,
// 	pub n_bins: u8, // max 256 bins
// 	offset: u8,
// }

// #[derive(Debug)]
// pub struct Feature {
// 	pub feature_index: usize,
// 	pub n_bins: u8,
// 	pub offset: u8,
// }

// impl FeatureBundle {
// 	fn new(feature_index: usize, binned_features: ArrayView1<u8>) -> Self {
// 		let has_nonzero_bin_value = binned_features.map(|bin| *bin != 0);
// 		let max_bin = binned_features.iter().max_by(|a, b| a.cmp(b)).unwrap();
// 		let n_bins = max_bin.to_u8().unwrap() + 1;
// 		let feature_bundle_entry = Feature {
// 			feature_index,
// 			n_bins,
// 			offset: 0,
// 		};
// 		Self {
// 			has_nonzero_bin_value,
// 			features: vec![feature_bundle_entry],
// 			offset: n_bins - 1,
// 			n_bins,
// 		}
// 	}
// 	fn add_feature(&mut self, feature_index: usize, binned_feature: ArrayView1<u8>) {
// 		// for each of the features, add it to the used_bins if the bin_index != 0
// 		Zip::from(self.has_nonzero_bin_value.as_slice_mut().unwrap())
// 			.and(binned_feature)
// 			.apply(|is_nonzero_bin_value, bin| {
// 				*is_nonzero_bin_value |= *bin != 0;
// 			});
// 		let max_bin = binned_feature.iter().max_by(|a, b| a.cmp(b)).unwrap();
// 		let n_bins = max_bin + 1;
// 		if n_bins == 1 {
// 			return;
// 		}
// 		let feature_group_entry = Feature {
// 			feature_index,
// 			n_bins,
// 			offset: self.offset,
// 		};
// 		self.features.push(feature_group_entry);
// 		self.n_bins += n_bins - 1;
// 		self.offset += n_bins - 1;
// 	}
// }

// // split categorical
// //https://github.com/microsoft/LightGBM/blob/25d149d8ceb92838dbf2f7331f9dc0dec701ffd8/src/treelearner/feature_histogram.hpp#L203
// // sort bins in a group by (sum_grad / sum_hessian + cat_smooth) where cat_smooth is by default 10.0 by lightgbm
// // min_data_per_group, minimum number of samples in this categorical group, default 100 in lightgbm

// #[test]
// fn test_bundled_binned_features() {
// 	let binned_features = arr2(&[
// 		[1, 0, 0, 0],
// 		[0, 1, 0, 0],
// 		[1, 0, 0, 0],
// 		[0, 0, 1, 0],
// 		[0, 0, 0, 0],
// 	]);
// 	// expect there to be one bin created
// 	let binning_thresholds = vec![vec![0.5], vec![0.5], vec![0.5], vec![]];
// 	let (feature_bundles, feature_to_feature_bundle, bundle_binning_thresholds) =
// 		compute_feature_bundles(binned_features.view(), binning_thresholds.as_slice());
// 	let left = compute_binned_feature_bundles(binned_features, feature_bundles.view());
// 	let right = arr2(&[[1], [2], [1], [3], [0]]);
// 	assert_eq!(left, right);
// }
