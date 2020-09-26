use self::{
	bin::{compute_binned_features, compute_binning_instructions},
	bin_stats::BinStatsPool,
	early_stopping::{compute_early_stopping_metric, EarlyStoppingMonitor},
	feature_importances::compute_feature_importances,
	tree::{
		TrainBranchNode, TrainBranchSplit, TrainBranchSplitContinuous, TrainBranchSplitDiscrete,
		TrainLeafNode, TrainNode,
	},
};
use crate::{
	binary_classifier::BinaryClassifier, multiclass_classifier::MulticlassClassifier,
	regressor::Regressor, BranchNode, BranchSplit, BranchSplitContinuous, BranchSplitDiscrete,
	LeafNode, Node, TrainOptions, TrainProgress, Tree,
};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use super_unsafe::SuperUnsafe;
use tangram_dataframe::*;
use tangram_progress::ProgressCounter;

mod bin;
mod bin_stats;
mod early_stopping;
mod examples_index;
mod feature_importances;
mod split;
mod tree;

pub use self::tree::TrainTree;

/// This enum is used by the common `train` function below to customize the training code slightly for each task.
#[derive(Clone, Copy, Debug)]
pub enum Task {
	Regression,
	BinaryClassification,
	MulticlassClassification { n_classes: usize },
}

/// This is the return type of the common `train` function.
#[derive(Debug)]
pub enum Model {
	Regressor(Regressor),
	BinaryClassifier(BinaryClassifier),
	MulticlassClassifier(MulticlassClassifier),
}

/// To avoid code duplication, this shared `train` function is called by `Regressor::train`, `BinaryClassifier::train`, and `MulticlassClassifier::train`.
pub fn train(
	task: Task,
	features: DataFrameView,
	labels: ColumnView,
	options: TrainOptions,
	update_progress: &mut dyn FnMut(TrainProgress),
) -> Model {
	#[cfg(feature = "timing")]
	let timing = super::timing::Timing::new();

	// If early stopping is enabled, split the features and labels into train and early stopping sets.
	let early_stopping_enabled = options.early_stopping_options.is_some();
	let (
		features_train,
		labels_train,
		features_early_stopping,
		labels_early_stopping,
		mut early_stopping_monitor,
	) = if let Some(early_stopping_options) = &options.early_stopping_options {
		let (features_train, labels_train, features_early_stopping, labels_early_stopping) =
			train_early_stopping_split(
				features,
				labels,
				early_stopping_options.early_stopping_fraction,
			);
		let early_stopping_monitor = EarlyStoppingMonitor::new(
			early_stopping_options.early_stopping_threshold,
			early_stopping_options.early_stopping_rounds,
		);
		(
			features_train,
			labels_train,
			Some(features_early_stopping.to_rows()),
			Some(labels_early_stopping),
			Some(early_stopping_monitor),
		)
	} else {
		(features, labels, None, None, None)
	};

	let n_features = features_train.ncols();
	let n_examples_train = features_train.nrows();

	// Determine how to bin each feature.
	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	let binning_instructions = compute_binning_instructions(&features_train, &options);
	#[cfg(feature = "timing")]
	timing.compute_binning_instructions.inc(start.elapsed());

	// Use the binning instructions from the previous step to compute the binned features.
	let progress_counter = ProgressCounter::new(features_train.nrows().to_u64().unwrap());
	update_progress(super::TrainProgress::Initializing(progress_counter.clone()));
	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	let binned_features = compute_binned_features(&features_train, &binning_instructions, &|| {
		progress_counter.inc(1)
	});
	#[cfg(feature = "timing")]
	timing.compute_binned_features.inc(start.elapsed());

	// Regression and binary classification train one tree per round. Multiclass classification trains one tree per class per round.
	let n_trees_per_round = match task {
		Task::Regression => 1,
		Task::BinaryClassification => 1,
		Task::MulticlassClassification { n_classes } => n_classes,
	};

	// The mean square error loss used in regression has a constant second derivative, so there is no need to use hessians for regression tasks.
	let has_constant_hessians = match task {
		Task::Regression => true,
		Task::BinaryClassification => false,
		Task::MulticlassClassification { .. } => false,
	};

	// Compute the biases. A tree model's prediction will be a bias plus the sum of the outputs of each tree. The bias will produce the baseline prediction.
	let biases = match task {
		// For regression, the bias is the mean of the labels.
		Task::Regression => {
			let labels_train = labels_train.as_number().unwrap().data.into();
			super::regressor::compute_biases(labels_train)
		}
		// For binary classification, the bias is the log of the ratio of positive examples to negative examples in the training set, so the baseline prediction is the majority class.
		Task::BinaryClassification => {
			let labels_train = labels_train.as_enum().unwrap().data.into();
			super::binary_classifier::compute_biases(labels_train)
		}
		// For multiclass classification the biases are the logs of each class's proporation in the training set, so the baseline prediction is the majority class.
		Task::MulticlassClassification { .. } => {
			let labels_train = labels_train.as_enum().unwrap().data.into();
			super::multiclass_classifier::compute_biases(labels_train, n_trees_per_round)
		}
	};

	// Pre-allocate memory to be used in training.
	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	let mut predictions = unsafe { Array::uninitialized((n_trees_per_round, n_examples_train)) };
	let mut gradients = unsafe { Array::uninitialized((n_trees_per_round, n_examples_train)) };
	let mut hessians = unsafe { Array::uninitialized((n_trees_per_round, n_examples_train)) };
	let mut ordered_gradients =
		unsafe { Array::uninitialized((n_trees_per_round, n_examples_train)) };
	let mut ordered_hessians =
		unsafe { Array::uninitialized((n_trees_per_round, n_examples_train)) };
	let mut examples_index = unsafe { Array::uninitialized((n_trees_per_round, n_examples_train)) };
	let mut examples_index_left_buffer =
		unsafe { Array::uninitialized((n_trees_per_round, n_examples_train)) };
	let mut examples_index_right_buffer =
		unsafe { Array::uninitialized((n_trees_per_round, n_examples_train)) };
	let mut bin_stats_pools: Vec<BinStatsPool> = Vec::with_capacity(n_trees_per_round);
	for _ in 0..n_trees_per_round {
		bin_stats_pools.push(BinStatsPool::new(
			options.max_leaf_nodes,
			&binning_instructions,
		));
	}
	let mut predictions_early_stopping = if early_stopping_enabled {
		let mut predictions_early_stopping = unsafe {
			Array::uninitialized((
				n_trees_per_round,
				labels_early_stopping.as_ref().unwrap().len(),
			))
		};
		for mut predictions in predictions_early_stopping.gencolumns_mut() {
			predictions.assign(&biases);
		}
		Some(predictions_early_stopping)
	} else {
		None
	};
	#[cfg(feature = "timing")]
	timing.allocations.inc(start.elapsed());

	// This is the total number of rounds that have been trained thus far.
	let mut n_rounds_trained = 0;
	// These are the trees in round-major order. After training this will be converted to an array of shape (n_rounds, n_trees_per_round).
	let mut trees: Vec<TrainTree> = Vec::new();
	// Collect the loss on the training dataset for each round if enabled.
	let mut losses: Option<Vec<f32>> = if options.compute_loss {
		Some(Vec::new())
	} else {
		None
	};

	// Before the first round, fill the predictions with the biases, which are the baseline predictions.
	for mut predictions_column in predictions.gencolumns_mut() {
		predictions_column.assign(&biases)
	}

	// Train rounds of trees until we hit max_rounds or the early stopping monitor indicates we should stop early.
	let round_counter = ProgressCounter::new(options.max_rounds.to_u64().unwrap());
	update_progress(super::TrainProgress::Training(round_counter.clone()));
	for _ in 0..options.max_rounds {
		round_counter.inc(1);
		// Before training the next round of trees, we need to determine what value for each example we would like the tree to learn.
		match task {
			Task::Regression => {
				let labels_train = labels_train.as_number().unwrap();
				super::regressor::compute_gradients_and_hessians(
					gradients.view_mut(),
					hessians.view_mut(),
					labels_train.data.into(),
					predictions.view(),
				);
			}
			Task::BinaryClassification => {
				let labels_train = labels_train.as_enum().unwrap();
				super::binary_classifier::compute_gradients_and_hessians(
					gradients.view_mut(),
					hessians.view_mut(),
					labels_train.data.into(),
					predictions.view(),
				);
			}
			Task::MulticlassClassification { .. } => {
				let labels_train = labels_train.as_enum().unwrap();
				super::multiclass_classifier::compute_gradients_and_hessians(
					gradients.view_mut(),
					hessians.view_mut(),
					labels_train.data.into(),
					predictions.view(),
				);
			}
		};
		// Train n_trees_per_round trees.
		let mut trees_for_round = Vec::with_capacity(n_trees_per_round);
		for (
			mut predictions,
			mut examples_index,
			mut examples_index_left_buffer,
			mut examples_index_right_buffer,
			gradients,
			hessians,
			mut ordered_gradients,
			mut ordered_hessians,
			bin_stats_pool,
		) in izip!(
			predictions.axis_iter_mut(Axis(0)),
			examples_index.axis_iter_mut(Axis(0)),
			examples_index_left_buffer.axis_iter_mut(Axis(0)),
			examples_index_right_buffer.axis_iter_mut(Axis(0)),
			gradients.axis_iter(Axis(0)),
			hessians.axis_iter(Axis(0)),
			ordered_gradients.axis_iter_mut(Axis(0)),
			ordered_hessians.axis_iter_mut(Axis(0)),
			bin_stats_pools.iter_mut(),
		) {
			// Reset the examples_index for this tree.
			for (index, value) in examples_index.iter_mut().enumerate() {
				*value = index;
			}
			// Train the tree.
			let tree = self::tree::train(
				&binned_features,
				gradients.as_slice().unwrap(),
				hessians.as_slice().unwrap(),
				ordered_gradients.as_slice_mut().unwrap(),
				ordered_hessians.as_slice_mut().unwrap(),
				examples_index.as_slice_mut().unwrap(),
				examples_index_left_buffer.as_slice_mut().unwrap(),
				examples_index_right_buffer.as_slice_mut().unwrap(),
				bin_stats_pool,
				has_constant_hessians,
				&options,
				#[cfg(feature = "timing")]
				&timing,
			);
			// Update the predictions using the leaf values from the most recently trained tree.
			#[cfg(feature = "timing")]
			let start = std::time::Instant::now();
			let predictions_cell = SuperUnsafe::new(predictions.as_slice_mut().unwrap());
			tree.leaf_values.iter().for_each(|(range, value)| {
				examples_index.as_slice().unwrap()[range.clone()]
					.iter()
					.for_each(|&example_index| {
						let predictions = unsafe { predictions_cell.get() };
						predictions[example_index] += value;
					});
			});
			#[cfg(feature = "timing")]
			timing.predict.inc(start.elapsed());
			trees_for_round.push(tree);
		}
		// If loss computation is enabled, compute the loss for this round.
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
		// If early stopping is enabled, compute the early stopping metric and update the early stopping monitor to see if we should stop training at this round.
		let should_stop = if early_stopping_enabled {
			let features_early_stopping = features_early_stopping.as_ref().unwrap();
			let labels_early_stopping = labels_early_stopping.as_ref().unwrap();
			let predictions_early_stopping = predictions_early_stopping.as_mut().unwrap();
			let early_stopping_monitor = early_stopping_monitor.as_mut().unwrap();
			let value = compute_early_stopping_metric(
				&task,
				trees_for_round.as_slice(),
				features_early_stopping.view(),
				labels_early_stopping.view(),
				predictions_early_stopping.view_mut(),
			);
			early_stopping_monitor.update(value)
		} else {
			false
		};
		// Add the trees for this round to the list of trees.
		trees.extend(trees_for_round);
		n_rounds_trained += 1;
		// Exit the training loop if we should stop.
		if should_stop {
			break;
		}
	}

	// Compute feature importances.
	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	let feature_importances = Some(compute_feature_importances(&trees, n_features));
	#[cfg(feature = "timing")]
	timing.compute_feature_importances.inc(start.elapsed());

	// Print out timing information if the timing feature is enabled.
	#[cfg(feature = "timing")]
	eprintln!("{:?}", timing);

	// Assemble the model.
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

/// Split the feature and labels into train and early stopping datasets, where the early stopping dataset with have `early_stopping_fraction * features.nrows()` rows.
fn train_early_stopping_split<'features, 'labels>(
	features: DataFrameView<'features>,
	labels: ColumnView<'labels>,
	early_stopping_fraction: f32,
) -> (
	DataFrameView<'features>,
	ColumnView<'labels>,
	DataFrameView<'features>,
	ColumnView<'labels>,
) {
	let split_index = (early_stopping_fraction * labels.len().to_f32().unwrap())
		.to_usize()
		.unwrap();
	let (features_early_stopping, features_train) = features.split_at_row(split_index);
	let (labels_early_stopping, labels_train) = labels.split_at_row(split_index);
	(
		features_train,
		labels_train,
		features_early_stopping,
		labels_early_stopping,
	)
}

impl From<TrainTree> for Tree {
	fn from(value: TrainTree) -> Tree {
		let nodes = value.nodes.into_iter().map(Into::into).collect();
		Tree { nodes }
	}
}

impl From<TrainNode> for Node {
	fn from(value: TrainNode) -> Node {
		match value {
			TrainNode::Branch(TrainBranchNode {
				left_child_index,
				right_child_index,
				split,
				examples_fraction,
				..
			}) => Node::Branch(BranchNode {
				left_child_index: left_child_index.unwrap(),
				right_child_index: right_child_index.unwrap(),
				split: match split {
					TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
						feature_index,
						invalid_values_direction,
						split_value,
						..
					}) => BranchSplit::Continuous(BranchSplitContinuous {
						feature_index,
						split_value,
						invalid_values_direction,
					}),
					TrainBranchSplit::Discrete(TrainBranchSplitDiscrete {
						feature_index,
						directions,
						..
					}) => BranchSplit::Discrete(BranchSplitDiscrete {
						feature_index,
						directions,
					}),
				},
				examples_fraction,
			}),
			TrainNode::Leaf(TrainLeafNode {
				value,
				examples_fraction,
			}) => Node::Leaf(LeafNode {
				value,
				examples_fraction,
			}),
		}
	}
}
