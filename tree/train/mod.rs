use self::{
	bin_stats::BinStatsPool,
	train_tree::{
		train_tree, TrainTreeBranchNode, TrainTreeBranchSplit, TrainTreeBranchSplitContinuous,
		TrainTreeBranchSplitDiscrete, TrainTreeLeafNode, TrainTreeNode,
	},
};
use crate::{
	binary_classifier::BinaryClassifier, multiclass_classifier::MulticlassClassifier,
	regressor::Regressor, BranchNode, BranchSplit, BranchSplitContinuous, BranchSplitDiscrete,
	LeafNode, Node, TrainOptions, TrainProgress, Tree,
};
use itertools::{izip, Itertools};
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::{cmp::Ordering, collections::BTreeMap};
use super_unsafe::SuperUnsafe;
use tangram_dataframe::*;
use tangram_finite::Finite;
use tangram_progress::ProgressCounter;

mod bin_stats;
mod choose_best_split;
mod rearrange_examples_index;
mod train_tree;

pub use self::train_tree::TrainTree;

/// This enum is used by the common `train` function below to customize the training code slightly for each task.
#[derive(Debug)]
pub enum Task {
	Regression,
	BinaryClassification,
	MulticlassClassification { n_trees_per_round: usize },
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
	task: &Task,
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
	timing
		.binning
		.compute_binning_instructions
		.inc(start.elapsed());

	// Use the binning instructions from the previous step to compute the binned features.
	let progress_counter = ProgressCounter::new(features_train.nrows().to_u64().unwrap());
	update_progress(super::TrainProgress::Initializing(progress_counter.clone()));
	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	let binned_features = compute_binned_features(&features_train, &binning_instructions, &|| {
		progress_counter.inc(1)
	});
	#[cfg(feature = "timing")]
	timing.binning.compute_binned_features.inc(start.elapsed());

	// Regression and binary classification train one tree per round. Multiclass classification trains one tree per class per round.
	let n_trees_per_round = match task {
		Task::Regression => 1,
		Task::BinaryClassification => 1,
		Task::MulticlassClassification { n_trees_per_round } => *n_trees_per_round,
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
	let mut examples_index_left =
		unsafe { Array::uninitialized((n_trees_per_round, n_examples_train)) };
	let mut examples_index_right =
		unsafe { Array::uninitialized((n_trees_per_round, n_examples_train)) };
	let mut bin_stats_pools: Vec<BinStatsPool> =
		vec![BinStatsPool::new(options.max_leaf_nodes, &binning_instructions); n_trees_per_round];
	let mut logits_early_stopping = if early_stopping_enabled {
		let mut logits_early_stopping = unsafe {
			Array::uninitialized((
				n_trees_per_round,
				labels_early_stopping.as_ref().unwrap().len(),
			))
		};
		for mut logits in logits_early_stopping.gencolumns_mut() {
			logits.assign(&biases);
		}
		Some(logits_early_stopping)
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
	for round_index in 0..options.max_rounds {
		round_counter.inc(1);
		// Before training the next round of trees, we need to determine which value for each example we would like the tree to learn.
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
				// Reset the examples_index.
				for (index, value) in examples_index.iter_mut().enumerate() {
					*value = index;
				}
				// Train the tree.
				let (tree, leaf_values) = train_tree(
					binned_features.as_slice(),
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
					#[cfg(feature = "timing")]
					&timing,
				);
				// Update the predictions with the most recently trained tree.
				if round_index < options.max_rounds - 1 {
					#[cfg(feature = "timing")]
					let start = std::time::Instant::now();
					let predictions_cell = SuperUnsafe::new(predictions.as_slice_mut().unwrap());
					leaf_values.iter().for_each(|(range, value)| {
						examples_index.as_slice().unwrap()[range.clone()]
							.iter()
							.for_each(|&example_index| {
								let predictions = unsafe { predictions_cell.get() };
								predictions[example_index] += value;
							});
					});
					#[cfg(feature = "timing")]
					timing.predict.inc(start.elapsed());
				}
				tree
			},
		)
		.collect::<Vec<_>>();
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
			let logits_early_stopping = logits_early_stopping.as_mut().unwrap();
			let early_stopping_monitor = early_stopping_monitor.as_mut().unwrap();
			let value = compute_early_stopping_metric(
				&task,
				trees_for_round.as_slice(),
				features_early_stopping.view(),
				labels_early_stopping.view(),
				logits_early_stopping.view_mut(),
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
	let trees: Vec<Tree> = trees.into_iter().map(tree_from_train_tree).collect();
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

/*
This struct specifies how to bin a feature.

## Number
Number features have the first bin reserved for invalid values, and after that feature values are binned by comparing them with a set of thresholds. For example, given the thresholds `[0.5, 1.5, 2]`, the bins will be:
0. invalid values
1. (-infinity, 0.5]
2. (0.5, 1.5]
3. (1.5, 2]
4. (2, infinity)

## Enum
Enum features have one bin for each enum option. For example, gives the options `["A", "B", "C"]`, the bins will be:
0. invalid values
1. "A"
2. "B"
3. "C"
*/
#[derive(Clone, Debug)]
pub enum BinningInstructions {
	Number { thresholds: Vec<f32> },
	Enum { n_options: usize },
}

impl BinningInstructions {
	pub fn n_valid_bins(&self) -> usize {
		match self {
			Self::Number { thresholds } => thresholds.len() + 1,
			Self::Enum { n_options } => *n_options,
		}
	}
}

/// Compute the binning instructions for each column in `features`.
pub fn compute_binning_instructions(
	features: &DataFrameView,
	options: &TrainOptions,
) -> Vec<BinningInstructions> {
	features
		.columns
		.iter()
		.map(|column| match column.view() {
			ColumnView::Number(column) => {
				compute_binning_instructions_for_number_feature(column, &options)
			}
			ColumnView::Enum(column) => BinningInstructions::Enum {
				n_options: column.options.len(),
			},
			_ => unreachable!(),
		})
		.collect()
}

/// Compute the binning instructions for a number feature.
fn compute_binning_instructions_for_number_feature(
	column: NumberColumnView,
	options: &TrainOptions,
) -> BinningInstructions {
	// Create a histogram of values in the number feature.
	let mut histogram: BTreeMap<Finite<f32>, usize> = BTreeMap::new();
	let mut histogram_values_count = 0;
	for value in &column.data[0..column
		.data
		.len()
		.min(options.max_examples_for_computing_bin_thresholds)]
	{
		if let Ok(value) = Finite::new(*value) {
			*histogram.entry(value).or_insert(0) += 1;
			histogram_values_count += 1;
		}
	}
	// If the number of unique values is less than `max_valid_bins_for_number_features`, then create one bin per unique value. Otherwise, create bins at quantiles.
	let thresholds = if histogram.len()
		< options
			.max_valid_bins_for_number_features
			.to_usize()
			.unwrap()
	{
		histogram
			.keys()
			.tuple_windows()
			.map(|(a, b)| (a.get() + b.get()) / 2.0)
			.collect()
	} else {
		compute_binning_instruction_thresholds_for_number_feature_as_quantiles_from_histogram(
			histogram,
			histogram_values_count,
			options,
		)
	};
	BinningInstructions::Number { thresholds }
}

/// Compute the binning instruction thresholds for a number feature as quantiles from the histogram of its values.
fn compute_binning_instruction_thresholds_for_number_feature_as_quantiles_from_histogram(
	histogram: BTreeMap<Finite<f32>, usize>,
	histogram_values_count: usize,
	options: &TrainOptions,
) -> Vec<f32> {
	let total_values_count = histogram_values_count.to_f32().unwrap();
	let quantiles: Vec<f32> = (1..options
		.max_valid_bins_for_number_features
		.to_usize()
		.unwrap())
		.map(|i| i.to_f32().unwrap() / options.max_valid_bins_for_number_features.to_f32().unwrap())
		.collect();
	let quantile_indexes: Vec<usize> = quantiles
		.iter()
		.map(|q| ((total_values_count - 1.0) * q).trunc().to_usize().unwrap())
		.collect();
	let quantile_fracts: Vec<f32> = quantiles
		.iter()
		.map(|q| ((total_values_count - 1.0) * q).fract())
		.collect();
	let mut quantiles: Vec<Option<f32>> = vec![None; quantiles.len()];
	let mut current_count: usize = 0;
	let mut iter = histogram.iter().peekable();
	while let Some((value, count)) = iter.next() {
		let value = value.get();
		current_count += count;
		let quantiles_iter = quantiles
			.iter_mut()
			.zip(quantile_indexes.iter().zip(quantile_fracts.iter()))
			.filter(|(q, (_, _))| q.is_none());
		for (quantile, (index, fract)) in quantiles_iter {
			match (current_count - 1).cmp(index) {
				Ordering::Equal => {
					if *fract > 0.0 {
						let next_value = iter.peek().unwrap().0.get();
						*quantile = Some(value * (1.0 - fract) + next_value * fract);
					} else {
						*quantile = Some(value);
					}
				}
				Ordering::Greater => *quantile = Some(value),
				Ordering::Less => {}
			}
		}
	}
	quantiles.into_iter().map(|q| q.unwrap()).collect()
}

pub enum BinnedFeaturesColumn {
	U8(Vec<u8>),
	U16(Vec<u16>),
}

/// Compute the binned features based on the binning instructions.
pub fn compute_binned_features(
	features: &DataFrameView,
	binning_instructions: &[BinningInstructions],
	progress: &(dyn Fn() + Sync),
) -> Vec<BinnedFeaturesColumn> {
	izip!(&features.columns, binning_instructions)
		.map(|(feature, binning_instructions)| {
			match binning_instructions {
				BinningInstructions::Number { thresholds } => {
					let binned_feature = feature
						.as_number()
						.unwrap()
						.data
						.iter()
						.map(|feature_value| {
							// Invalid values go to the first bin.
							if !feature_value.is_finite() {
								return 0;
							}
							// Use binary search on the thresholds to find the bin for the feature value.
							thresholds
								.binary_search_by(|threshold| {
									threshold.partial_cmp(feature_value).unwrap()
								})
								.unwrap_or_else(|bin| bin)
								.to_u8()
								.unwrap() + 1
						})
						.collect::<Vec<u8>>();
					progress();
					BinnedFeaturesColumn::U8(binned_feature)
				}
				BinningInstructions::Enum { n_options } => {
					// TODO
					if *n_options <= 15 {
						let binned_feature = feature
							.as_enum()
							.unwrap()
							.data
							.iter()
							.map(|feature_value| {
								feature_value.map(|v| v.get()).unwrap_or(0).to_u8().unwrap()
							})
							.collect::<Vec<u8>>();
						progress();
						BinnedFeaturesColumn::U8(binned_feature)
					} else if *n_options <= 255 {
						let binned_feature = feature
							.as_enum()
							.unwrap()
							.data
							.iter()
							.map(|feature_value| {
								feature_value.map(|v| v.get()).unwrap_or(0).to_u8().unwrap()
							})
							.collect::<Vec<u8>>();
						progress();
						BinnedFeaturesColumn::U8(binned_feature)
					} else {
						let binned_feature = feature
							.as_enum()
							.unwrap()
							.data
							.iter()
							.map(|feature_value| {
								feature_value
									.map(|v| v.get())
									.unwrap_or(0)
									.to_u16()
									.unwrap()
							})
							.collect::<Vec<u16>>();
						progress();
						BinnedFeaturesColumn::U16(binned_feature)
					}
				}
			}
		})
		.collect()
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

/// Compute the early stopping metric value for the set of trees that have been trained thus far.
fn compute_early_stopping_metric(
	task: &Task,
	trees: &[TrainTree],
	features: ArrayView2<Value>,
	labels: ColumnView,
	mut logits: ArrayViewMut2<f32>,
) -> f32 {
	match task {
		Task::Regression => {
			let labels = labels.as_number().unwrap().data.into();
			super::regressor::update_logits(trees, features.view(), logits.view_mut());
			super::regressor::compute_loss(labels, logits.view())
		}
		Task::BinaryClassification => {
			let labels = labels.as_enum().unwrap().data.into();
			super::binary_classifier::update_logits(trees, features.view(), logits.view_mut());
			super::binary_classifier::compute_loss(labels, logits.view())
		}
		Task::MulticlassClassification { .. } => {
			let labels = labels.as_enum().unwrap().data.into();
			super::multiclass_classifier::update_logits(trees, features.view(), logits.view_mut());
			super::multiclass_classifier::compute_loss(labels, logits.view())
		}
	}
}

#[derive(Clone)]
struct EarlyStoppingMonitor {
	tolerance: f32,
	max_rounds_no_improve: usize,
	previous_stopping_metric: Option<f32>,
	num_rounds_no_improve: usize,
}

impl EarlyStoppingMonitor {
	/// Create a train stop monitor,
	pub fn new(tolerance: f32, max_rounds_no_improve: usize) -> Self {
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

/// This function computes feature importances using the "split" method, where a feature's importance is proportional to the number of nodes that use it to split.
fn compute_feature_importances(trees: &[TrainTree], n_features: usize) -> Vec<f32> {
	let mut feature_importances = vec![0.0; n_features];
	for tree in trees.iter() {
		for node in tree.nodes.iter() {
			match node {
				TrainTreeNode::Branch(TrainTreeBranchNode {
					split:
						TrainTreeBranchSplit::Continuous(TrainTreeBranchSplitContinuous {
							feature_index,
							..
						}),
					..
				})
				| TrainTreeNode::Branch(TrainTreeBranchNode {
					split:
						TrainTreeBranchSplit::Discrete(TrainTreeBranchSplitDiscrete {
							feature_index,
							..
						}),
					..
				}) => {
					feature_importances[*feature_index] += 1.0;
				}
				TrainTreeNode::Leaf(_) => {}
			}
		}
	}
	// Normalize the feature_importances.
	let total = feature_importances.iter().sum::<f32>();
	for feature_importance in feature_importances.iter_mut() {
		*feature_importance /= total;
	}
	feature_importances
}

fn tree_from_train_tree(tree: TrainTree) -> Tree {
	let nodes = tree
		.nodes
		.into_iter()
		.map(|node| match node {
			TrainTreeNode::Branch(TrainTreeBranchNode {
				left_child_index,
				right_child_index,
				split,
				examples_fraction,
				..
			}) => Node::Branch(BranchNode {
				left_child_index: left_child_index.unwrap(),
				right_child_index: right_child_index.unwrap(),
				split: match split {
					TrainTreeBranchSplit::Continuous(TrainTreeBranchSplitContinuous {
						feature_index,
						invalid_values_direction,
						split_value,
						..
					}) => BranchSplit::Continuous(BranchSplitContinuous {
						feature_index,
						split_value,
						invalid_values_direction,
					}),
					TrainTreeBranchSplit::Discrete(TrainTreeBranchSplitDiscrete {
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
			TrainTreeNode::Leaf(TrainTreeLeafNode {
				value,
				examples_fraction,
			}) => Node::Leaf(LeafNode {
				value,
				examples_fraction,
			}),
		})
		.collect();
	Tree { nodes }
}
