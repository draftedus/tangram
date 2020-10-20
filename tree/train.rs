#[cfg(feature = "timing")]
use crate::timing::Timing;
use crate::{
	binary_classifier::BinaryClassifier,
	compute_bin_stats::{BinStats, BinStatsEntry},
	compute_binned_features::{
		compute_binned_features_column_major, compute_binned_features_row_major,
	},
	compute_binning_instructions::compute_binning_instructions,
	compute_feature_importances::compute_feature_importances,
	multiclass_classifier::MulticlassClassifier,
	regressor::Regressor,
	train_tree::{
		train_tree, TrainBranchNode, TrainBranchSplit, TrainBranchSplitContinuous,
		TrainBranchSplitDiscrete, TrainLeafNode, TrainNode, TrainTree, TrainTreeOptions,
	},
	BinnedFeaturesLayout, BranchNode, BranchSplit, BranchSplitContinuous, BranchSplitDiscrete,
	LeafNode, Node, TrainOptions, TrainProgress, Tree,
};
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use rayon::prelude::*;
use tangram_dataframe::prelude::*;
use tangram_util::{pool::Pool, progress_counter::ProgressCounter, super_unsafe::SuperUnsafe};

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
	labels: DataFrameColumnView,
	train_options: TrainOptions,
	update_progress: &mut dyn FnMut(TrainProgress),
) -> Model {
	#[cfg(feature = "timing")]
	let timing = Timing::new();

	// If early stopping is enabled, split the features and labels into train and early stopping sets.
	let early_stopping_enabled = train_options.early_stopping_options.is_some();
	let (
		features_train,
		labels_train,
		features_early_stopping,
		labels_early_stopping,
		mut early_stopping_monitor,
	) = if let Some(early_stopping_options) = &train_options.early_stopping_options {
		let (features_train, labels_train, features_early_stopping, labels_early_stopping) =
			train_early_stopping_split(
				features,
				labels,
				early_stopping_options.early_stopping_fraction,
			);
		let early_stopping_monitor = EarlyStoppingMonitor::new(
			early_stopping_options.min_decrease_in_loss_for_significant_change,
			early_stopping_options.n_epochs_without_improvement_to_stop,
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
	let binning_instructions = compute_binning_instructions(&features_train, &train_options);
	#[cfg(feature = "timing")]
	timing.compute_binning_instructions.inc(start.elapsed());

	// Use the binning instructions from the previous step to compute the binned features.
	let binned_features_layout = train_options.binned_features_layout;
	let progress_counter = ProgressCounter::new(features_train.nrows().to_u64().unwrap());
	update_progress(TrainProgress::Initializing(progress_counter.clone()));
	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	let binned_features_row_major =
		if let BinnedFeaturesLayout::RowMajor = train_options.binned_features_layout {
			Some(compute_binned_features_row_major(
				&features_train,
				&binning_instructions,
				&|| progress_counter.inc(1),
			))
		} else {
			None
		};
	let binned_features_column_major =
		compute_binned_features_column_major(&features_train, &binning_instructions, &|| {
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
	let hessians_are_constant = match task {
		Task::Regression => true,
		Task::BinaryClassification => false,
		Task::MulticlassClassification { .. } => false,
	};

	// Compute the biases. A tree model's prediction will be a bias plus the sum of the outputs of each tree. The bias will produce the baseline prediction.
	let biases = match task {
		// For regression, the bias is the mean of the labels.
		Task::Regression => {
			let labels_train = labels_train.as_number().unwrap();
			let labels_train = labels_train.as_slice().into();
			crate::regressor::compute_biases(labels_train)
		}
		// For binary classification, the bias is the log of the ratio of positive examples to negative examples in the training set, so the baseline prediction is the majority class.
		Task::BinaryClassification => {
			let labels_train = labels_train.as_enum().unwrap();
			let labels_train = labels_train.as_slice().into();
			crate::binary_classifier::compute_biases(labels_train)
		}
		// For multiclass classification the biases are the logs of each class's proporation in the training set, so the baseline prediction is the majority class.
		Task::MulticlassClassification { .. } => {
			let labels_train = labels_train.as_enum().unwrap();
			let labels_train = labels_train.as_slice().into();
			crate::multiclass_classifier::compute_biases(labels_train, n_trees_per_round)
		}
	};

	// Pre-allocate memory to be used in training.
	let mut predictions =
		unsafe { Array::uninitialized((n_examples_train, n_trees_per_round).f()) };
	let mut gradients = unsafe { Array::uninitialized(n_examples_train) };
	let mut hessians = unsafe { Array::uninitialized(n_examples_train) };
	let mut gradients_ordered_buffer = unsafe { Array::uninitialized(n_examples_train) };
	let mut hessians_ordered_buffer = unsafe { Array::uninitialized(n_examples_train) };
	let mut examples_index = unsafe { Array::uninitialized(n_examples_train) };
	let mut examples_index_left_buffer = unsafe { Array::uninitialized(n_examples_train) };
	let mut examples_index_right_buffer = unsafe { Array::uninitialized(n_examples_train) };
	let mut predictions_early_stopping = if early_stopping_enabled {
		let mut predictions_early_stopping = unsafe {
			Array::uninitialized((
				n_trees_per_round,
				labels_early_stopping.as_ref().unwrap().len(),
			))
		};
		for mut predictions in predictions_early_stopping.axis_iter_mut(Axis(1)) {
			predictions.assign(&biases);
		}
		Some(predictions_early_stopping)
	} else {
		None
	};
	let binning_instructions_for_pool = binning_instructions.clone();
	let bin_stats_pool = match binned_features_layout {
		BinnedFeaturesLayout::ColumnMajor => Pool::new(
			train_options.max_leaf_nodes,
			Box::new(move || {
				BinStats::ColumnMajor(
					binning_instructions_for_pool
						.iter()
						.map(|binning_instructions| {
							vec![BinStatsEntry::default(); binning_instructions.n_bins()]
						})
						.collect(),
				)
			}),
		),
		BinnedFeaturesLayout::RowMajor => Pool::new(
			train_options.max_leaf_nodes,
			Box::new(move || {
				BinStats::RowMajor(
					binning_instructions_for_pool
						.iter()
						.flat_map(|binning_instructions| {
							vec![BinStatsEntry::default(); binning_instructions.n_bins()]
						})
						.collect(),
				)
			}),
		),
	};

	// This is the total number of rounds that have been trained thus far.
	let mut n_rounds_trained = 0;
	// These are the trees in round-major order. After training this will be converted to an array of shape (n_rounds, n_trees_per_round).
	let mut trees: Vec<TrainTree> = Vec::new();
	// Collect the loss on the training dataset for each round if enabled.
	let mut losses: Option<Vec<f32>> = if train_options.compute_loss {
		Some(Vec::new())
	} else {
		None
	};

	// Before the first round, fill the predictions with the biases, which are the baseline predictions.
	for mut predictions in predictions.axis_iter_mut(Axis(0)) {
		predictions.assign(&biases)
	}

	// Train rounds of trees until we hit max_rounds or the early stopping monitor indicates we should stop early.
	let round_counter = ProgressCounter::new(train_options.max_rounds.to_u64().unwrap());
	update_progress(TrainProgress::Training(round_counter.clone()));
	for _ in 0..train_options.max_rounds {
		round_counter.inc(1);
		// Train n_trees_per_round trees.
		let mut trees_for_round = Vec::with_capacity(n_trees_per_round);
		for tree_per_round_index in 0..n_trees_per_round {
			// Before training the next tree, we need to determine what value for each example we would like the tree to learn.
			#[cfg(feature = "timing")]
			let start = std::time::Instant::now();
			match task {
				Task::Regression => {
					let labels_train = labels_train.as_number().unwrap();
					crate::regressor::compute_gradients_and_hessians(
						gradients.as_slice_mut().unwrap(),
						hessians.as_slice_mut().unwrap(),
						labels_train.as_slice(),
						predictions.column(0).as_slice().unwrap(),
					);
				}
				Task::BinaryClassification => {
					let labels_train = labels_train.as_enum().unwrap();
					crate::binary_classifier::compute_gradients_and_hessians(
						gradients.as_slice_mut().unwrap(),
						hessians.as_slice_mut().unwrap(),
						labels_train.as_slice(),
						predictions.column(0).as_slice().unwrap(),
					);
				}
				Task::MulticlassClassification { .. } => {
					let labels_train = labels_train.as_enum().unwrap();
					crate::multiclass_classifier::compute_gradients_and_hessians(
						tree_per_round_index,
						gradients.as_slice_mut().unwrap(),
						hessians.as_slice_mut().unwrap(),
						labels_train.as_slice(),
						predictions.view(),
					);
				}
			};
			#[cfg(feature = "timing")]
			timing.compute_gradients_and_hessians.inc(start.elapsed());
			// Reset the examples_index.
			examples_index
				.as_slice_mut()
				.unwrap()
				.par_iter_mut()
				.enumerate()
				.for_each(|(index, value)| {
					*value = index.to_u32().unwrap();
				});
			// Train the tree.
			let tree = train_tree(TrainTreeOptions {
				binning_instructions: &binning_instructions,
				binned_features_row_major: &binned_features_row_major,
				binned_features_column_major: &binned_features_column_major,
				gradients: gradients.as_slice().unwrap(),
				hessians: hessians.as_slice().unwrap(),
				gradients_ordered_buffer: gradients_ordered_buffer.as_slice_mut().unwrap(),
				hessians_ordered_buffer: hessians_ordered_buffer.as_slice_mut().unwrap(),
				examples_index: examples_index.as_slice_mut().unwrap(),
				examples_index_left_buffer: examples_index_left_buffer.as_slice_mut().unwrap(),
				examples_index_right_buffer: examples_index_right_buffer.as_slice_mut().unwrap(),
				bin_stats_pool: &bin_stats_pool,
				hessians_are_constant,
				train_options: &train_options,
				#[cfg(feature = "timing")]
				timing: &timing,
			});
			// Update the predictions using the leaf values from the tree.
			update_predictions_with_tree(
				predictions
					.column_mut(tree_per_round_index)
					.as_slice_mut()
					.unwrap(),
				examples_index.as_slice().unwrap(),
				&tree,
				#[cfg(feature = "timing")]
				&timing,
			);
			trees_for_round.push(tree);
		}
		// If loss computation is enabled, compute the loss for this round.
		if let Some(losses) = losses.as_mut() {
			let loss = match task {
				Task::Regression => {
					let labels_train = labels_train.as_number().unwrap();
					let labels_train = labels_train.as_slice().into();
					crate::regressor::compute_loss(labels_train, predictions.view())
				}
				Task::BinaryClassification => {
					let labels_train = labels_train.as_enum().unwrap();
					let labels_train = labels_train.as_slice().into();
					crate::binary_classifier::compute_loss(labels_train, predictions.view())
				}
				Task::MulticlassClassification { .. } => {
					let labels_train = labels_train.as_enum().unwrap();
					let labels_train = labels_train.as_slice().into();
					crate::multiclass_classifier::compute_loss(labels_train, predictions.view())
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

	// Compute the feature importances.
	let feature_importances = Some(compute_feature_importances(&trees, n_features));

	// Print out the timing and tree information if the timing feature is enabled.
	#[cfg(feature = "timing")]
	eprintln!("{:?}", timing);

	// Assemble the model.
	let trees: Vec<Tree> = trees.into_iter().map(Into::into).collect();
	match task {
		Task::Regression => Model::Regressor(Regressor {
			bias: *biases.get(0).unwrap(),
			trees,
			feature_importances,
			train_options,
		}),
		Task::BinaryClassification => Model::BinaryClassifier(BinaryClassifier {
			bias: *biases.get(0).unwrap(),
			trees,
			feature_importances,
			train_options,
		}),
		Task::MulticlassClassification { .. } => {
			Model::MulticlassClassifier(MulticlassClassifier {
				n_rounds: n_rounds_trained,
				n_classes: n_trees_per_round,
				biases: biases.into_raw_vec(),
				trees,
				feature_importances,
				train_options,
			})
		}
	}
}

fn update_predictions_with_tree(
	predictions: &mut [f32],
	examples_index: &[u32],
	tree: &TrainTree,
	#[cfg(feature = "timing")] timing: &Timing,
) {
	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	let predictions_cell = SuperUnsafe::new(predictions);
	tree.leaf_values.par_iter().for_each(|(range, value)| {
		examples_index[range.clone()]
			.iter()
			.for_each(|example_index| unsafe {
				let example_index = example_index.to_usize().unwrap();
				*predictions_cell.get().get_unchecked_mut(example_index) += value;
			});
	});
	#[cfg(feature = "timing")]
	timing.update_predictions.inc(start.elapsed());
}

#[derive(Clone)]
pub struct EarlyStoppingMonitor {
	tolerance: f32,
	max_rounds_no_improve: usize,
	previous_stopping_metric: Option<f32>,
	num_rounds_no_improve: usize,
}

impl EarlyStoppingMonitor {
	/// Create a train stop monitor,
	pub fn new(tolerance: f32, max_rounds_no_improve: usize) -> EarlyStoppingMonitor {
		EarlyStoppingMonitor {
			tolerance,
			max_rounds_no_improve,
			previous_stopping_metric: None,
			num_rounds_no_improve: 0,
		}
	}

	/// Update with the next epoch's task metrics. Returns true if training should stop.
	pub fn update(&mut self, value: f32) -> bool {
		let stopping_metric = value;
		let result = if let Some(previous_stopping_metric) = self.previous_stopping_metric {
			if stopping_metric > previous_stopping_metric
				|| f32::abs(stopping_metric - previous_stopping_metric) < self.tolerance
			{
				self.num_rounds_no_improve += 1;
				self.num_rounds_no_improve >= self.max_rounds_no_improve
			} else {
				self.num_rounds_no_improve = 0;
				false
			}
		} else {
			false
		};
		self.previous_stopping_metric = Some(stopping_metric);
		result
	}
}

/// Split the feature and labels into train and early stopping datasets, where the early stopping dataset will have `early_stopping_fraction * features.nrows()` rows.
fn train_early_stopping_split<'features, 'labels>(
	features: DataFrameView<'features>,
	labels: DataFrameColumnView<'labels>,
	early_stopping_fraction: f32,
) -> (
	DataFrameView<'features>,
	DataFrameColumnView<'labels>,
	DataFrameView<'features>,
	DataFrameColumnView<'labels>,
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

/// Compute the early stopping metric value for the set of trees that have been trained thus far.
fn compute_early_stopping_metric(
	task: &Task,
	trees_for_round: &[TrainTree],
	features: ArrayView2<DataFrameValue>,
	labels: DataFrameColumnView,
	mut predictions: ArrayViewMut2<f32>,
) -> f32 {
	match task {
		Task::Regression => {
			let labels = labels.as_number().unwrap();
			let labels = labels.as_slice().into();
			crate::regressor::update_logits(
				trees_for_round,
				features.view(),
				predictions.view_mut(),
			);
			crate::regressor::compute_loss(labels, predictions.view())
		}
		Task::BinaryClassification => {
			let labels = labels.as_enum().unwrap();
			let labels = labels.as_slice().into();
			crate::binary_classifier::update_logits(
				trees_for_round,
				features.view(),
				predictions.view_mut(),
			);
			crate::binary_classifier::compute_loss(labels, predictions.view())
		}
		Task::MulticlassClassification { .. } => {
			let labels = labels.as_enum().unwrap();
			let labels = labels.as_slice().into();
			crate::multiclass_classifier::update_logits(
				trees_for_round,
				features.view(),
				predictions.view_mut(),
			);
			crate::multiclass_classifier::compute_loss(labels, predictions.view())
		}
	}
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
				..
			}) => Node::Leaf(LeafNode {
				value,
				examples_fraction,
			}),
		}
	}
}
