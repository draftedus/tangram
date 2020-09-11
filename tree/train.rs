use super::{
	bin::{
		compute_bin_info, compute_binned_features, filter_binned_features, ComputeBinInfoOptions,
		FilterBinnedFeaturesOptions,
	},
	early_stopping::{
		compute_early_stopping_metrics, train_early_stopping_split, TrainStopMonitor,
	},
	single,
	single::bin_stats::BinStatsPool,
	*,
};
use crate::util::progress_counter::ProgressCounter;
use crate::{dataframe::*, util::super_unsafe::SuperUnsafe};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::ops::Range;

/// Train a gradient boosted decision tree model.
pub fn train(
	task: &Task,
	features: DataFrameView,
	labels: ColumnView,
	options: TrainOptions,
	update_progress: &mut dyn FnMut(super::Progress),
) -> Model {
	// let timing = timing::Timing::new();

	// determine how to bin each column
	let bin_options = ComputeBinInfoOptions {
		max_valid_bins: options.max_non_missing_bins,
		max_number_column_examples_for_bin_info: options.subsample_for_binning,
	};

	let bin_info = compute_bin_info(&features, &bin_options);

	// compute the binned features
	let n_bins = options.max_non_missing_bins as usize + 1;
	let progress_counter = ProgressCounter::new(features.nrows().to_u64().unwrap());
	update_progress(super::Progress::Initializing(progress_counter.clone()));
	let (features, features_stats) =
		compute_binned_features(&features, &bin_info, n_bins as usize, &|| {
			progress_counter.inc(1)
		});

	let filter_options = FilterBinnedFeaturesOptions {
		min_examples_split: options.min_examples_leaf,
	};
	let include_features =
		filter_binned_features(features.view(), features_stats, &bin_info, filter_options);

	let (features_train, labels_train) = (features, labels);

	let early_stopping_options = &options.early_stopping_options;

	struct EarlyStopping<'features, 'labels> {
		train_stop_monitor: TrainStopMonitor,
		features_early_stopping: ArrayView2<'features, u8>,
		labels_early_stopping: ColumnView<'labels>,
	};

	let (features_train, labels_train, mut early_stopping) = match early_stopping_options {
		Some(options) => {
			// if early stopping is enabled then split the features
			// and labels into train and early stopping sets.
			let (features_train, labels_train, features_early_stopping, labels_early_stopping) =
				train_early_stopping_split(
					features_train.view(),
					labels_train,
					options.early_stopping_fraction,
				);
			(
				features_train.to_owned(),
				labels_train,
				Some(EarlyStopping {
					train_stop_monitor: TrainStopMonitor::new(
						options.early_stopping_threshold,
						options.early_stopping_rounds,
					),
					features_early_stopping,
					labels_early_stopping,
				}),
			)
		}
		None => (features_train, labels_train, None),
	};

	let n_examples = features_train.nrows();
	let n_features = features_train.ncols();

	// regression and binary classification have one tree for each round,
	// multiclass classification has one tree per class for each round.
	let n_trees_per_round = match task {
		Task::Regression => 1,
		Task::BinaryClassification => 1,
		Task::MulticlassClassification { n_trees_per_round } => *n_trees_per_round,
	};

	// The mean square error loss used in regression has a constant second derivative,
	// so there is no need to update hessians for regression tasks.
	let has_constant_hessians = match task {
		Task::Regression => true,
		Task::BinaryClassification => false,
		Task::MulticlassClassification { .. } => false,
	};

	// A Tree model's prediction will be a bias plus the sum of the outputs of each tree.
	// The bias will produce the baseline prediction.
	let biases = match task {
		// For regression, the baseline prediction is the mean of the labels.
		Task::Regression => {
			let labels_train = labels_train.as_number().unwrap().data.into();
			super::regressor::compute_biases(labels_train)
		}
		// For binary classification, the bias is the log of the ratio of positive examples
		// to negative examples in the training set, so the baseline prediction is the majority class.
		Task::BinaryClassification => {
			let labels_train = labels_train.as_enum().unwrap().data.into();
			super::binary_classifier::compute_biases(labels_train)
		}
		// For multiclass classification the biases are the logs of each class's
		// proporation in the training set, so the baseline prediction is the majority class.
		Task::MulticlassClassification { .. } => {
			let labels_train = labels_train.as_enum().unwrap().data.into();
			super::multiclass_classifier::compute_biases(labels_train, n_trees_per_round)
		}
	};

	// Pre-allocate memory to be used in training.
	let mut predictions = unsafe { Array::uninitialized((n_trees_per_round, n_examples)) };
	for mut predictions_column in predictions.gencolumns_mut() {
		predictions_column.assign(&biases)
	}
	let (mut gradients, mut hessians, mut ordered_gradients, mut ordered_hessians) = (
		unsafe { Array::uninitialized((n_trees_per_round, n_examples)) },
		unsafe { Array::uninitialized((n_trees_per_round, n_examples)) },
		unsafe { Array::uninitialized((n_trees_per_round, n_examples)) },
		unsafe { Array::uninitialized((n_trees_per_round, n_examples)) },
	);
	let mut examples_index = unsafe { Array::uninitialized((n_trees_per_round, n_examples)) };
	let mut examples_index_left = unsafe { Array::uninitialized((n_trees_per_round, n_examples)) };
	let mut examples_index_right = unsafe { Array::uninitialized((n_trees_per_round, n_examples)) };
	let mut bin_stats_pools: Vec<BinStatsPool> =
		vec![BinStatsPool::new(options.max_leaf_nodes, &bin_info); n_trees_per_round];

	// This is the total number of rounds that have been trained thus far.
	let mut n_rounds_trained = 0;
	// These are the trees in round-major order. After training this will
	// This will have shape (n_rounds, n_trees_per_round).
	let mut trees: Vec<single::TrainTree> = Vec::new();
	// Collect the loss on the training dataset for each round if enabled.
	let mut losses: Option<Vec<f32>> = if options.compute_loss {
		Some(Vec::new())
	} else {
		None
	};

	let progress_counter = ProgressCounter::new(options.max_rounds.to_u64().unwrap());
	update_progress(super::Progress::Training(progress_counter.clone()));
	// This is the primary training loop.
	for round_index in 0..options.max_rounds {
		progress_counter.inc(1);
		// Update the gradients and hessians before each iteration. In the first iteration we update the gradients and hessians using the loss computed between the baseline prediction and the labels.
		match task {
			Task::Regression => {
				let labels_train = labels_train.as_number().unwrap();
				super::regressor::update_gradients_and_hessians(
					gradients.view_mut(),
					hessians.view_mut(),
					labels_train.data.into(),
					predictions.view(),
				);
			}
			Task::BinaryClassification => {
				let labels_train = labels_train.as_enum().unwrap();
				super::binary_classifier::update_gradients_and_hessians(
					gradients.view_mut(),
					hessians.view_mut(),
					labels_train.data.into(),
					predictions.view(),
				);
			}
			Task::MulticlassClassification { .. } => {
				let labels_train = labels_train.as_enum().unwrap();
				super::multiclass_classifier::update_gradients_and_hessians(
					gradients.view_mut(),
					hessians.view_mut(),
					labels_train.data.into(),
					predictions.view(),
				);
			}
		};
		// Train n_trees_per_round trees in parallel.
		let trees_for_round = izip!(
			predictions.axis_iter_mut(Axis(0)),
			examples_index.axis_iter_mut(Axis(0)),
			examples_index_left.axis_iter_mut(Axis(0)),
			examples_index_right.axis_iter_mut(Axis(0)),
			gradients.axis_iter(Axis(0)),
			hessians.axis_iter(Axis(0)),
			ordered_gradients.axis_iter_mut(Axis(0)),
			ordered_hessians.axis_iter_mut(Axis(0)),
			bin_stats_pools.iter_mut(),
		)
		.map(
			|(
				mut predictions,
				mut examples_index,
				mut examples_index_left,
				mut examples_index_right,
				gradients,
				hessians,
				mut ordered_gradients,
				mut ordered_hessians,
				bin_stats_pool,
			)| {
				// reset the examples_index to sorted order
				for (index, value) in examples_index.iter_mut().enumerate() {
					*value = index;
				}
				// train the tree
				let (tree, leaf_values) = single::train::train(
					features_train.view(),
					&include_features,
					gradients.as_slice().unwrap(),
					hessians.as_slice().unwrap(),
					ordered_gradients.as_slice_mut().unwrap(),
					ordered_hessians.as_slice_mut().unwrap(),
					examples_index.as_slice_mut().unwrap(),
					examples_index_left.as_slice_mut().unwrap(),
					examples_index_right.as_slice_mut().unwrap(),
					bin_stats_pool,
					has_constant_hessians,
					&options,
				);
				// update the predictions with the most recently trained tree
				if round_index < options.max_rounds - 1 {
					update_predictions_from_leaves(
						&leaf_values,
						predictions.as_slice_mut().unwrap(),
						examples_index.as_slice().unwrap(),
					);
				}
				tree
			},
		)
		.collect::<Vec<_>>();

		// If loss computation was enabled, then compute the loss for this round.
		if let Some(losses) = losses.as_mut() {
			let loss = match task {
				Task::Regression => {
					let labels_train = labels_train.as_number().unwrap().data.into();
					super::regressor::compute_loss(labels_train, predictions.view())
				}
				Task::BinaryClassification => {
					let labels_train = labels_train.as_enum().unwrap().data.into();
					super::binary_classifier::compute_loss(labels_train, predictions.view())
				}
				Task::MulticlassClassification { .. } => {
					let labels_train = labels_train.as_enum().unwrap().data.into();
					super::multiclass_classifier::compute_loss(labels_train, predictions.view())
				}
			};
			losses.push(loss);
		}

		if let Some(early_stopping) = &mut early_stopping {
			let n_examples_early_stopping = early_stopping.features_early_stopping.nrows();
			let mut logits_early_stopping =
				{ Array::zeros((n_trees_per_round, n_examples_early_stopping)) };
			for mut logits in logits_early_stopping.gencolumns_mut() {
				logits.assign(&biases);
			}

			// compute the early stopping metrics and update the train stop monitor
			// to see if we should stop training at this round.
			let value = compute_early_stopping_metrics(
				&task,
				trees_for_round.as_slice(),
				early_stopping.features_early_stopping,
				early_stopping.labels_early_stopping.clone(),
				logits_early_stopping.view_mut(),
			);
			let should_stop = early_stopping.train_stop_monitor.update(value);
			if should_stop {
				// add the trees for this round to the list of trees.
				trees.extend(trees_for_round);
				n_rounds_trained += 1;
				break;
			}
		}

		// add the trees for this round to the list of trees.
		trees.extend(trees_for_round);
		n_rounds_trained += 1;
	}

	// convert losses to ndarray
	let losses = losses.map(Into::into);

	// compute feature importances
	let feature_importances = Some(compute_feature_importances(&trees, n_features));

	// assemble the model
	let trees: Vec<Tree> = trees.into_iter().map(Into::into).collect();
	match task {
		Task::Regression => Model::Regressor(Regressor {
			bias: biases[0],
			trees,
			feature_importances,
			losses,
		}),
		Task::BinaryClassification => {
			let classes = match labels_train {
				ColumnView::Enum(c) => c.options.to_vec(),
				_ => unreachable!(),
			};
			Model::BinaryClassifier(BinaryClassifier {
				bias: biases[0],
				trees,
				feature_importances,
				losses,
				classes,
			})
		}
		Task::MulticlassClassification { .. } => {
			let classes = match labels_train {
				ColumnView::Enum(c) => c.options.to_vec(),
				_ => unreachable!(),
			};
			Model::MulticlassClassifier(MulticlassClassifier {
				n_rounds: n_rounds_trained,
				n_classes: n_trees_per_round,
				biases: biases.into_raw_vec(),
				trees,
				feature_importances,
				losses,
				classes,
			})
		}
	}
}

/// Update predictions by traversing the leaf nodes
pub fn update_predictions_from_leaves(
	leaf_values: &[(Range<usize>, f32)],
	predictions: &mut [f32],
	examples_index: &[usize],
) {
	let predictions_cell = SuperUnsafe::new(predictions);
	leaf_values.iter().for_each(|(range, value)| {
		examples_index[range.clone()]
			.iter()
			.for_each(|&example_index| {
				let predictions = unsafe { predictions_cell.get() };
				predictions[example_index] += value;
			});
	});
}

/**
This function computes feature importances using the "split" method, where a feature's importance is proportional to the number of nodes that use it to split.
*/
fn compute_feature_importances(trees: &[single::TrainTree], n_features: usize) -> Vec<f32> {
	let mut feature_importances = vec![0.0; n_features];
	for tree in trees.iter() {
		tree.nodes.iter().for_each(|node| match node {
			single::TrainNode::Branch(single::TrainBranchNode {
				split:
					single::TrainBranchSplit::Continuous(single::TrainBranchSplitContinuous {
						feature_index,
						..
					}),
				..
			})
			| single::TrainNode::Branch(single::TrainBranchNode {
				split:
					single::TrainBranchSplit::Discrete(single::TrainBranchSplitDiscrete {
						feature_index,
						..
					}),
				..
			}) => {
				feature_importances[*feature_index] += 1.0;
			}
			single::TrainNode::Leaf(_) => {}
		});
	}
	// Normalize the feature_importances.
	let total: f32 = feature_importances.iter().sum();
	for feature_importance in feature_importances.iter_mut() {
		*feature_importance /= total;
	}
	feature_importances
}
