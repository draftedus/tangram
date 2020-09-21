use crate::{
	binary_classifier::BinaryClassifier, multiclass_classifier::MulticlassClassifier,
	regressor::Regressor, BranchNode, BranchSplit, BranchSplitContinuous, BranchSplitDiscrete,
	LeafNode, Node, SplitDirection, TrainOptions, TrainProgress, Tree,
};
use itertools::{izip, Itertools};
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::{cmp::Ordering, collections::BTreeMap, collections::BinaryHeap, ops::Range};
use super_unsafe::SuperUnsafe;
use tangram_dataframe::*;
use tangram_finite::Finite;
use tangram_progress::ProgressCounter;

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
	let mut trees: Vec<SingleTree> = Vec::new();
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
	let trees: Vec<Tree> = trees.into_iter().map(tree_from_single_tree).collect();
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
	trees: &[SingleTree],
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
fn compute_feature_importances(trees: &[SingleTree], n_features: usize) -> Vec<f32> {
	let mut feature_importances = vec![0.0; n_features];
	for tree in trees.iter() {
		for node in tree.nodes.iter() {
			match node {
				SingleTreeNode::Branch(SingleTreeBranchNode {
					split:
						SingleTreeBranchSplit::Continuous(SingleTreeBranchSplitContinuous {
							feature_index,
							..
						}),
					..
				})
				| SingleTreeNode::Branch(SingleTreeBranchNode {
					split:
						SingleTreeBranchSplit::Discrete(SingleTreeBranchSplitDiscrete {
							feature_index,
							..
						}),
					..
				}) => {
					feature_importances[*feature_index] += 1.0;
				}
				SingleTreeNode::Leaf(_) => {}
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

fn tree_from_single_tree(tree: SingleTree) -> Tree {
	let nodes = tree
		.nodes
		.into_iter()
		.map(|node| match node {
			SingleTreeNode::Branch(SingleTreeBranchNode {
				left_child_index,
				right_child_index,
				split,
				examples_fraction,
				..
			}) => Node::Branch(BranchNode {
				left_child_index: left_child_index.unwrap(),
				right_child_index: right_child_index.unwrap(),
				split: match split {
					SingleTreeBranchSplit::Continuous(SingleTreeBranchSplitContinuous {
						feature_index,
						invalid_values_direction,
						split_value,
						..
					}) => BranchSplit::Continuous(BranchSplitContinuous {
						feature_index,
						split_value,
						invalid_values_direction,
					}),
					SingleTreeBranchSplit::Discrete(SingleTreeBranchSplitDiscrete {
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
			SingleTreeNode::Leaf(SingleTreeLeafNode {
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

#[derive(Debug)]
pub struct SingleTree {
	pub nodes: Vec<SingleTreeNode>,
}

#[derive(Debug)]
pub enum SingleTreeNode {
	Branch(SingleTreeBranchNode),
	Leaf(SingleTreeLeafNode),
}

impl SingleTreeNode {
	pub fn as_branch_mut(&mut self) -> Option<&mut SingleTreeBranchNode> {
		match self {
			SingleTreeNode::Branch(s) => Some(s),
			_ => None,
		}
	}
}

#[derive(Debug)]
pub struct SingleTreeBranchNode {
	pub left_child_index: Option<usize>,
	pub right_child_index: Option<usize>,
	pub split: SingleTreeBranchSplit,
	pub examples_fraction: f32,
}

#[derive(Clone, Debug)]
pub enum SingleTreeBranchSplit {
	Continuous(SingleTreeBranchSplitContinuous),
	Discrete(SingleTreeBranchSplitDiscrete),
}

#[derive(Clone, Debug)]
pub struct SingleTreeBranchSplitContinuous {
	pub feature_index: usize,
	pub split_value: f32,
	pub bin_index: u8,
	pub invalid_values_direction: SplitDirection,
}

#[derive(Clone, Debug)]
pub struct SingleTreeBranchSplitDiscrete {
	pub feature_index: usize,
	pub directions: Vec<bool>,
}

#[derive(Debug)]
pub struct SingleTreeLeafNode {
	pub value: f32,
	pub examples_fraction: f32,
}

struct QueueItem {
	/// Items in the priority queue will be sorted by the gain of the split.
	pub gain: f32,
	/// A split describes how the node is split into left and right children.
	pub split: SingleTreeBranchSplit,
	/// The queue item holds a reference to its parent so that
	/// it can update the parent's left or right child index
	/// if the queue item becomes a node added to the tree.
	pub parent_index: Option<usize>,
	/// Will this node be a left or right child of its parent?
	pub split_direction: Option<SplitDirection>,
	/// The depth of the item in the tree.
	pub depth: usize,
	/// The bin_stats consisting of aggregate hessian/gradient statistics of the training examples that reach this node.
	pub bin_stats: BinStats,
	/// The examples_index_range tells you what the range of
	/// examples indexes in the examples_index specifies
	/// the examples in this node.
	pub examples_index_range: std::ops::Range<usize>,
	/// The sum of the gradients of all of the training examples in this node.
	pub sum_gradients: f64,
	/// The sum of the hessians of all of the training examples in this node.
	pub sum_hessians: f64,
	/// The sum of the gradients of all of the training examples that go to the left child.
	pub left_sum_gradients: f64,
	/// The sum of the hessians of all of the training examples that go to the left child.
	pub left_sum_hessians: f64,
	/// The total number of training examples that go to the left child.
	pub left_n_examples: usize,
	/// The sum of the gradients of all of the training examples that go to the right child.
	pub right_sum_gradients: f64,
	/// The sum of the hessians of all of the training examples that go to the right child.
	pub right_sum_hessians: f64,
	/// The total number of training examples that go to the right child.
	pub right_n_examples: usize,
}

impl PartialEq for QueueItem {
	fn eq(&self, other: &Self) -> bool {
		self.gain == other.gain
	}
}

impl Eq for QueueItem {}

impl std::cmp::PartialOrd for QueueItem {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.gain.partial_cmp(&other.gain)
	}
}

impl std::cmp::Ord for QueueItem {
	fn cmp(&self, other: &Self) -> Ordering {
		self.partial_cmp(&other).unwrap()
	}
}

impl SingleTree {
	/// Make a prediction for a given example.
	pub fn predict(&self, features: ArrayView1<tangram_dataframe::Value>) -> f32 {
		// Start at the root node.
		let mut node_index = 0;
		loop {
			match &self.nodes[node_index] {
				// We are at a branch, decide whether to send this example to the left or right child.
				SingleTreeNode::Branch(SingleTreeBranchNode {
					left_child_index,
					right_child_index,
					split,
					..
				}) => match split {
					// This branch uses a continuous split.
					SingleTreeBranchSplit::Continuous(SingleTreeBranchSplitContinuous {
						feature_index,
						split_value,
						..
					}) => {
						node_index = if features[*feature_index].as_number().unwrap() <= split_value
						{
							left_child_index.unwrap()
						} else {
							right_child_index.unwrap()
						};
					}
					// This branch uses a discrete split.
					SingleTreeBranchSplit::Discrete(SingleTreeBranchSplitDiscrete {
						feature_index,
						directions,
						..
					}) => {
						let bin_index =
							if let Some(bin_index) = features[*feature_index].as_enum().unwrap() {
								bin_index.get()
							} else {
								0
							};
						node_index = if !directions.get(bin_index).unwrap() {
							left_child_index.unwrap()
						} else {
							right_child_index.unwrap()
						};
					}
				},
				// We made it to a leaf! The prediction is the leaf's value.
				SingleTreeNode::Leaf(SingleTreeLeafNode { value, .. }) => return *value,
			}
		}
	}
}

#[derive(Clone)]
pub struct BinStats {
	/// One bin info per feature
	pub binning_instructions: Vec<BinningInstructions>,
	/// (n_features)
	pub entries: Vec<Vec<f64>>,
}

impl BinStats {
	pub fn new(binning_instructions: Vec<BinningInstructions>) -> Self {
		let entries = binning_instructions
			.iter()
			.map(|b| vec![0.0; 2 * (b.n_valid_bins() + 1)])
			.collect();
		Self {
			binning_instructions,
			entries,
		}
	}
}

#[derive(Clone)]
pub struct BinStatsPool {
	pub items: Vec<BinStats>,
}

impl BinStatsPool {
	pub fn new(size: usize, binning_instructions: &[BinningInstructions]) -> Self {
		let mut items = Vec::with_capacity(size);
		for _ in 0..size {
			items.push(BinStats::new(binning_instructions.to_owned()));
		}
		Self { items }
	}
	pub fn get(&mut self) -> BinStats {
		self.items.pop().unwrap()
	}
}

/// Trains a single tree.
#[allow(clippy::too_many_arguments)]
pub fn train_tree(
	binned_features: &[BinnedFeaturesColumn],
	gradients: &[f32],
	hessians: &[f32],
	ordered_gradients: &mut [f32],
	ordered_hessians: &mut [f32],
	examples_index: &mut [usize],
	examples_index_left: &mut [usize],
	examples_index_right: &mut [usize],
	bin_stats_pool: &mut BinStatsPool,
	hessians_are_constant: bool,
	options: &TrainOptions,
	#[cfg(feature = "timing")] timing: &crate::timing::Timing,
) -> (SingleTree, Vec<(Range<usize>, f32)>) {
	// This is the tree returned by this function
	let mut tree = SingleTree { nodes: Vec::new() };
	// This priority queue stores the potential nodes to split ordered by their gain.
	let mut queue: BinaryHeap<QueueItem> = BinaryHeap::new();
	// To update the gradients and hessians we need to make predictions.
	// Rather than running each example through the tree, we can reuse
	// the mapping from example index to leaf value previously computed.
	let mut leaf_values: Vec<(Range<usize>, f32)> = Vec::new();

	// Compute the sums of gradients and hessians for the root node.
	let n_examples = gradients.len();
	let examples_index_range = 0..n_examples;
	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	let sum_gradients = gradients.iter().map(|v| v.to_f64().unwrap()).sum();
	let sum_hessians = if hessians_are_constant {
		n_examples.to_f64().unwrap()
	} else {
		hessians.iter().map(|v| v.to_f64().unwrap()).sum()
	};
	#[cfg(feature = "timing")]
	timing.sum_gradients_hessians.inc(start.elapsed());

	// If there are too few training examples or the hessians are too small,
	// just return a tree with a single leaf.
	if n_examples < 2 * options.min_examples_per_child
		|| sum_hessians < 2.0 * options.min_sum_hessians_per_child.to_f64().unwrap()
	{
		let value = compute_leaf_value(sum_gradients, sum_hessians, options);
		let node = SingleTreeNode::Leaf(SingleTreeLeafNode {
			value,
			examples_fraction: 1.0,
		});
		tree.nodes.push(node);
		leaf_values.push((examples_index_range, value));
		return (tree, leaf_values);
	}

	// compute the bin stats for the root node
	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	let mut root_bin_stats = bin_stats_pool.get();
	compute_bin_stats_for_root_node(
		&mut root_bin_stats,
		binned_features,
		gradients,
		hessians,
		hessians_are_constant,
	);
	#[cfg(feature = "timing")]
	timing.bin_stats.compute_bin_stats_root.inc(start.elapsed());

	// based on the node stats and bin stats, find a split, if any.
	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	let find_split_output = find_split(
		&root_bin_stats,
		sum_gradients,
		sum_hessians,
		examples_index_range.clone(),
		&options,
	);
	#[cfg(feature = "timing")]
	timing.find_split.inc(start.elapsed());

	// If we were able to find a split for the root node, add it to the queue and proceed to the loop. Otherwise, return a tree with a single node.
	if let Some(find_split_output) = find_split_output {
		queue.push(QueueItem {
			depth: 0,
			examples_index_range,
			gain: find_split_output.gain,
			left_n_examples: find_split_output.left_n_examples,
			left_sum_gradients: find_split_output.left_sum_gradients,
			left_sum_hessians: find_split_output.left_sum_hessians,
			bin_stats: root_bin_stats,
			parent_index: None,
			right_n_examples: find_split_output.right_n_examples,
			right_sum_gradients: find_split_output.right_sum_gradients,
			right_sum_hessians: find_split_output.right_sum_hessians,
			split_direction: None,
			split: find_split_output.split,
			sum_gradients,
			sum_hessians,
		});
	} else {
		let value = compute_leaf_value(sum_gradients, sum_hessians, options);
		let examples_count = examples_index_range.len();
		leaf_values.push((examples_index_range, value));
		let node = SingleTreeNode::Leaf(SingleTreeLeafNode {
			value,
			examples_fraction: examples_count.to_f32().unwrap() / n_examples.to_f32().unwrap(),
		});
		tree.nodes.push(node);
		// Return the bin stats to the pool.
		bin_stats_pool.items.push(root_bin_stats);
		return (tree, leaf_values);
	}

	while let Some(queue_item) = queue.pop() {
		// Update the node's parent left or right child index with the current node index
		// There are two cases:
		// 1. The current node's split direction is left: it is a left child of its parent
		// 2. The current node's split_direction is right: it is a right child of its parent
		let node_index = tree.nodes.len();
		if let Some(parent_index) = queue_item.parent_index {
			let parent = tree
				.nodes
				.get_mut(parent_index)
				.unwrap()
				.as_branch_mut()
				.unwrap();
			let split_direction = queue_item.split_direction.unwrap();
			match split_direction {
				SplitDirection::Left => parent.left_child_index = Some(node_index),
				SplitDirection::Right => parent.right_child_index = Some(node_index),
			}
		}

		// Determine the current number of leaf nodes if training were to stop now. If the max leaf nodes is reached, add the current node as a leaf and continue until all items are removed from the queue and added to the tree as leaves
		let n_leaf_nodes = leaf_values.len() + queue.len() + 1;
		let max_leaf_nodes_reached = n_leaf_nodes == options.max_leaf_nodes;
		if max_leaf_nodes_reached {
			let value =
				compute_leaf_value(queue_item.sum_gradients, queue_item.sum_hessians, options);
			let examples_count = queue_item.examples_index_range.len();
			let node = SingleTreeNode::Leaf(SingleTreeLeafNode {
				value,
				examples_fraction: examples_count.to_f32().unwrap() / n_examples.to_f32().unwrap(),
			});
			leaf_values.push((queue_item.examples_index_range.clone(), value));
			tree.nodes.push(node);
			bin_stats_pool.items.push(queue_item.bin_stats);
			continue;
		}

		// TODO
		// Add the current node to the tree. The missing values direction is the direction with more training examples. TODO: This is the naive implementation that does not compute whether sending missing values to the left subtree or right subtree results in a higher gain. Instead, we simply send missing values in the direction where the majority of training examples go.
		let missing_values_direction = if queue_item.left_n_examples > queue_item.right_n_examples {
			SplitDirection::Left
		} else {
			SplitDirection::Right
		};

		tree.nodes
			.push(SingleTreeNode::Branch(SingleTreeBranchNode {
				split: queue_item.split.clone(),
				left_child_index: None,
				right_child_index: None,
				examples_fraction: queue_item.examples_index_range.len().to_f32().unwrap()
					/ n_examples.to_f32().unwrap(),
			}));

		// Rearrange the examples index.
		#[cfg(feature = "timing")]
		let start = std::time::Instant::now();
		let (left, right) = rearrange_examples_index(
			binned_features,
			&queue_item.split,
			examples_index
				.get_mut(queue_item.examples_index_range.clone())
				.unwrap(),
			examples_index_left
				.get_mut(queue_item.examples_index_range.clone())
				.unwrap(),
			examples_index_right
				.get_mut(queue_item.examples_index_range.clone())
				.unwrap(),
		);
		#[cfg(feature = "timing")]
		timing.rearrange_examples_index.inc(start.elapsed());

		// The left and right ranges are local to the node, so add the node's start to make them global.
		let start = queue_item.examples_index_range.start;
		let left_examples_index_range = start + left.start..start + left.end;
		let right_examples_index_range = start + right.start..start + right.end;

		// Determine if we should split left and/or right based on the number of examples in the node and the node's depth in the tree.
		let max_depth_reached = queue_item.depth + 1 == options.max_depth;
		let should_split_left = !max_depth_reached
			&& left_examples_index_range.len() >= options.min_examples_per_child * 2;
		let should_split_right = !max_depth_reached
			&& right_examples_index_range.len() >= options.min_examples_per_child * 2;

		// If we should not split left, add a leaf.
		if !should_split_left {
			let left_child_index = tree.nodes.len();
			let value = compute_leaf_value(
				queue_item.left_sum_gradients,
				queue_item.left_sum_hessians,
				options,
			);
			let node = SingleTreeNode::Leaf(SingleTreeLeafNode {
				value,
				examples_fraction: queue_item.left_n_examples.to_f32().unwrap()
					/ n_examples.to_f32().unwrap(),
			});
			leaf_values.push((left_examples_index_range.clone(), value));
			tree.nodes.push(node);
			// Set the parent's child index to the new node's index.
			let parent = tree
				.nodes
				.get_mut(node_index)
				.unwrap()
				.as_branch_mut()
				.unwrap();
			parent.left_child_index = Some(left_child_index);
		}

		// If we should not split right, add a leaf.
		if !should_split_right {
			let right_child_index = tree.nodes.len();
			let value = compute_leaf_value(
				queue_item.right_sum_gradients,
				queue_item.right_sum_hessians,
				options,
			);
			let node = SingleTreeNode::Leaf(SingleTreeLeafNode {
				value,
				examples_fraction: queue_item.right_n_examples.to_f32().unwrap()
					/ n_examples.to_f32().unwrap(),
			});
			leaf_values.push((right_examples_index_range.clone(), value));
			tree.nodes.push(node);
			// Set the parent's child index to the new node's index.
			let parent = tree
				.nodes
				.get_mut(node_index)
				.unwrap()
				.as_branch_mut()
				.unwrap();
			parent.right_child_index = Some(right_child_index);
		}

		// If we should not split either left or right, then there is nothing left to do, so we can go to the next item on the queue.
		if !should_split_left && !should_split_right {
			// Return the bin stats to the pool.
			bin_stats_pool.items.push(queue_item.bin_stats);
			continue;
		}

		// Next, we compute the bin stats for the two children. `smaller_direction` is the direction of the child with fewer examples.
		let smaller_direction =
			if left_examples_index_range.len() < right_examples_index_range.len() {
				SplitDirection::Left
			} else {
				SplitDirection::Right
			};
		let smaller_child_examples_index = match smaller_direction {
			SplitDirection::Left => &examples_index[left_examples_index_range.clone()],
			SplitDirection::Right => &examples_index[right_examples_index_range.clone()],
		};
		let mut smaller_child_bin_stats = bin_stats_pool.get();

		// Compute the bin stats for the child with fewer examples.
		#[cfg(feature = "timing")]
		let start = std::time::Instant::now();
		compute_bin_stats_for_non_root_node(
			&mut smaller_child_bin_stats,
			ordered_gradients,
			ordered_hessians,
			binned_features,
			gradients,
			hessians,
			hessians_are_constant,
			smaller_child_examples_index,
		);
		#[cfg(feature = "timing")]
		timing.bin_stats.compute_bin_stats.inc(start.elapsed());

		// Compute the bin stats for the child with more examples by subtracting the bin stats of the child with fewer examples from the parent's bin stats.
		#[cfg(feature = "timing")]
		let start = std::time::Instant::now();
		let mut larger_child_bin_stats = queue_item.bin_stats;
		compute_bin_stats_subtraction(&mut larger_child_bin_stats, &smaller_child_bin_stats);
		#[cfg(feature = "timing")]
		timing
			.bin_stats
			.compute_bin_stats_subtraction
			.inc(start.elapsed());
		let (left_bin_stats, right_bin_stats) = match smaller_direction {
			SplitDirection::Left => (smaller_child_bin_stats, larger_child_bin_stats),
			SplitDirection::Right => (larger_child_bin_stats, smaller_child_bin_stats),
		};

		// If both left and right should split, find the splits for both at the same
		// time. Allows for a slight speedup because of cache. TODO: this speedup is probably not there.
		#[cfg(feature = "timing")]
		let start = std::time::Instant::now();
		let (left_find_split_output, right_find_split_output) =
			if should_split_left && should_split_right {
				// based on the node stats and bin stats, find a split, if any.
				let (left_find_split_output, right_find_split_output) = find_split_both(
					&left_bin_stats,
					queue_item.left_sum_gradients,
					queue_item.left_sum_hessians,
					left_examples_index_range.clone(),
					&right_bin_stats,
					queue_item.right_sum_gradients,
					queue_item.right_sum_hessians,
					right_examples_index_range.clone(),
					&options,
				);
				(left_find_split_output, right_find_split_output)
			} else if should_split_left {
				// Based on the node stats and bin stats, find a split, if any.
				let find_split_output = find_split(
					&left_bin_stats,
					queue_item.left_sum_gradients,
					queue_item.left_sum_hessians,
					left_examples_index_range.clone(),
					&options,
				);
				(find_split_output, None)
			} else if should_split_right {
				// Based on the node stats and bin stats, find a split, if any.
				let find_split_output = find_split(
					&right_bin_stats,
					queue_item.right_sum_gradients,
					queue_item.right_sum_hessians,
					right_examples_index_range.clone(),
					&options,
				);
				(None, find_split_output)
			} else {
				(None, None)
			};
		#[cfg(feature = "timing")]
		timing.find_split.inc(start.elapsed());

		// If we were able to find a split for the node, add it to the queue. Otherwise, add a leaf.
		if should_split_left {
			if let Some(find_split_output) = left_find_split_output {
				queue.push(QueueItem {
					depth: queue_item.depth + 1,
					examples_index_range: left_examples_index_range.clone(),
					gain: find_split_output.gain,
					left_n_examples: find_split_output.left_n_examples,
					left_sum_gradients: find_split_output.left_sum_gradients,
					left_sum_hessians: find_split_output.left_sum_hessians,
					bin_stats: left_bin_stats,
					parent_index: Some(node_index),
					right_n_examples: find_split_output.right_n_examples,
					right_sum_gradients: find_split_output.right_sum_gradients,
					right_sum_hessians: find_split_output.right_sum_hessians,
					split_direction: Some(SplitDirection::Left),
					split: find_split_output.split,
					sum_gradients: queue_item.left_sum_gradients,
					sum_hessians: queue_item.left_sum_hessians,
				});
			} else {
				let left_child_index = tree.nodes.len();
				let value = compute_leaf_value(sum_gradients, sum_hessians, options);
				leaf_values.push((left_examples_index_range, value));
				let node = SingleTreeNode::Leaf(SingleTreeLeafNode {
					value,
					examples_fraction: queue_item.left_n_examples.to_f32().unwrap()
						/ n_examples.to_f32().unwrap(),
				});
				tree.nodes.push(node);
				// Set the parent's left child index to the new node's index.
				let parent = tree
					.nodes
					.get_mut(node_index)
					.unwrap()
					.as_branch_mut()
					.unwrap();
				parent.left_child_index = Some(left_child_index);
				// Return the bin stats to the pool.
				bin_stats_pool.items.push(left_bin_stats);
			}
		} else {
			bin_stats_pool.items.push(left_bin_stats);
		}

		// If we were able to find a split for the node, add it to the queue. Otherwise, add a leaf.
		if should_split_right {
			if let Some(find_split_output) = right_find_split_output {
				queue.push(QueueItem {
					depth: queue_item.depth + 1,
					examples_index_range: right_examples_index_range.clone(),
					gain: find_split_output.gain,
					left_n_examples: find_split_output.left_n_examples,
					left_sum_gradients: find_split_output.left_sum_gradients,
					left_sum_hessians: find_split_output.left_sum_hessians,
					bin_stats: right_bin_stats,
					parent_index: Some(node_index),
					right_n_examples: find_split_output.right_n_examples,
					right_sum_gradients: find_split_output.right_sum_gradients,
					right_sum_hessians: find_split_output.right_sum_hessians,
					split_direction: Some(SplitDirection::Right),
					split: find_split_output.split,
					sum_gradients: queue_item.right_sum_gradients,
					sum_hessians: queue_item.right_sum_hessians,
				});
			} else {
				let right_child_index = tree.nodes.len();
				let value = compute_leaf_value(sum_gradients, sum_hessians, options);
				leaf_values.push((right_examples_index_range, value));
				let node = SingleTreeNode::Leaf(SingleTreeLeafNode {
					value,
					examples_fraction: queue_item.right_n_examples.to_f32().unwrap()
						/ n_examples.to_f32().unwrap(),
				});
				tree.nodes.push(node);
				// Set the parent's left child index to the new node's index.
				let parent = tree
					.nodes
					.get_mut(node_index)
					.unwrap()
					.as_branch_mut()
					.unwrap();
				parent.right_child_index = Some(right_child_index);
				// Return the bin stats to the pool.
				bin_stats_pool.items.push(right_bin_stats);
			}
		} else {
			bin_stats_pool.items.push(right_bin_stats)
		}
	}

	(SingleTree { nodes: tree.nodes }, leaf_values)
}

/// Compute the value for a leaf node.
#[inline(always)]
fn compute_leaf_value(sum_gradients: f64, sum_hessians: f64, options: &TrainOptions) -> f32 {
	(-options.learning_rate.to_f64().unwrap() * sum_gradients
		/ (sum_hessians + options.l2_regularization.to_f64().unwrap() + std::f64::EPSILON))
		.to_f32()
		.unwrap()
}

/// This static control how far ahead in the `examples_index` the `compute_bin_stats_*` functions should prefetch binned_features to be used in subsequent iterations.
#[cfg(target_arch = "x86_64")]
static PREFETCH_OFFSET: usize = 64;

/// This static control how many times to unroll the loop in `compute_bin_stats_for_feature_not_root`.
static ROOT_UNROLL: usize = 16;

/// This static control how many times to unroll the loop in `compute_bin_stats_for_feature_not_root`.
static NOT_ROOT_UNROLL: usize = 4;

pub enum BinnedFeaturesColumn {
	U8(Vec<u8>),
	U16(Vec<u16>),
}

pub fn compute_bin_stats_for_root_node(
	node_bin_stats: &mut BinStats,
	// (n_examples, n_features) column major
	binned_features: &[BinnedFeaturesColumn],
	// (n_examples)
	gradients: &[f32],
	// (n_examples)
	hessians: &[f32],
	// hessians are constant in least squares loss, so we don't have to waste time updating them
	hessians_are_constant: bool,
) {
	izip!(&mut node_bin_stats.entries, binned_features.iter(),).for_each(
		|(bin_stats_for_feature, binned_feature_values)| {
			for entry in bin_stats_for_feature.iter_mut() {
				*entry = 0.0;
			}
			if hessians_are_constant {
				unsafe {
					match binned_feature_values {
						BinnedFeaturesColumn::U8(binned_feature_values) => {
							compute_bin_stats_for_feature_root_no_hessian(
								gradients,
								binned_feature_values,
								bin_stats_for_feature,
							)
						}
						BinnedFeaturesColumn::U16(binned_feature_values) => {
							compute_bin_stats_for_feature_root_no_hessian(
								gradients,
								binned_feature_values,
								bin_stats_for_feature,
							)
						}
					}
				}
			} else {
				unsafe {
					match binned_feature_values {
						BinnedFeaturesColumn::U8(binned_feature_values) => {
							compute_bin_stats_for_feature_root(
								gradients,
								hessians,
								binned_feature_values,
								bin_stats_for_feature,
							)
						}
						BinnedFeaturesColumn::U16(binned_feature_values) => {
							compute_bin_stats_for_feature_root(
								gradients,
								hessians,
								binned_feature_values,
								bin_stats_for_feature,
							)
						}
					}
				};
			}
		},
	);
}

#[allow(clippy::collapsible_if)]
#[allow(clippy::too_many_arguments)]
pub fn compute_bin_stats_for_non_root_node(
	node_bin_stats: &mut BinStats,
	// (n_examples)
	ordered_gradients: &mut [f32],
	// (n_examples)
	ordered_hessians: &mut [f32],
	// (n_examples, n_features) column major
	binned_features: &[BinnedFeaturesColumn],
	// (n_examples)
	gradients: &[f32],
	// (n_examples)
	hessians: &[f32],
	// hessians are constant in least squares loss, so we don't have to waste time updating them
	hessians_are_constant: bool,
	// these are the indexes of the examples at this node, length only equal to n_examples at the root node
	examples_index_for_node: &[usize],
) {
	let n_examples_in_node = examples_index_for_node.len();
	if !hessians_are_constant {
		for i in 0..n_examples_in_node {
			ordered_gradients[i] = gradients[examples_index_for_node[i]];
			ordered_hessians[i] = hessians[examples_index_for_node[i]]
		}
	} else {
		for i in 0..n_examples_in_node {
			ordered_gradients[i] = gradients[examples_index_for_node[i]];
		}
	}
	izip!(&mut node_bin_stats.entries, binned_features.iter(),).for_each(
		|(bin_stats_for_feature, binned_feature_values)| {
			for entry in bin_stats_for_feature.iter_mut() {
				*entry = 0.0;
			}
			if hessians_are_constant {
				unsafe {
					match binned_feature_values {
						BinnedFeaturesColumn::U8(binned_feature_values) => {
							compute_bin_stats_for_feature_not_root_no_hessians(
								ordered_gradients,
								binned_feature_values.as_slice(),
								bin_stats_for_feature,
								examples_index_for_node,
							)
						}
						BinnedFeaturesColumn::U16(binned_feature_values) => {
							compute_bin_stats_for_feature_not_root_no_hessians(
								ordered_gradients,
								binned_feature_values.as_slice(),
								bin_stats_for_feature,
								examples_index_for_node,
							)
						}
					}
				}
			} else {
				unsafe {
					match binned_feature_values {
						BinnedFeaturesColumn::U8(binned_feature_values) => {
							compute_bin_stats_for_feature_not_root(
								ordered_gradients,
								ordered_hessians,
								binned_feature_values.as_slice(),
								bin_stats_for_feature,
								examples_index_for_node,
							)
						}
						BinnedFeaturesColumn::U16(binned_feature_values) => {
							compute_bin_stats_for_feature_not_root(
								ordered_gradients,
								ordered_hessians,
								binned_feature_values.as_slice(),
								bin_stats_for_feature,
								examples_index_for_node,
							)
						}
					}
				}
			}
		},
	);
}

unsafe fn compute_bin_stats_for_feature_root_no_hessian<T>(
	gradients: &[f32],
	binned_feature_values: &[T],
	bin_stats_for_feature: &mut [f64],
) where
	T: num_traits::cast::ToPrimitive,
{
	let unroll = ROOT_UNROLL;
	let len = gradients.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			let ordered_gradient = *gradients.get_unchecked(i);
			let bin_index = binned_feature_values.get_unchecked(i).to_usize().unwrap();
			let bin_index = bin_index << 1;
			*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
			*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += 1.0;
		}
	}
	for i in (len / unroll) * unroll..len {
		let ordered_gradient = *gradients.get_unchecked(i);
		let bin_index = binned_feature_values.get_unchecked(i).to_usize().unwrap();
		let bin_index = bin_index << 1;
		*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
		*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += 1.0;
	}
}

pub unsafe fn compute_bin_stats_for_feature_root<T>(
	gradients: &[f32],
	hessians: &[f32],
	binned_feature_values: &[T],
	bin_stats_for_feature: &mut [f64],
) where
	T: ToPrimitive,
{
	let unroll = ROOT_UNROLL;
	let len = gradients.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			let ordered_gradient = *gradients.get_unchecked(i);
			let ordered_hessian = *hessians.get_unchecked(i);
			let bin_index = binned_feature_values.get_unchecked(i).to_usize().unwrap();
			let bin_index = bin_index << 1;
			*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
			*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += ordered_hessian as f64;
		}
	}
	for i in (len / unroll) * unroll..len {
		let ordered_gradient = *gradients.get_unchecked(i);
		let ordered_hessian = *hessians.get_unchecked(i);
		let bin_index = binned_feature_values.get_unchecked(i).to_usize().unwrap();
		let bin_index = bin_index << 1;
		*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
		*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += ordered_hessian as f64;
	}
}

unsafe fn compute_bin_stats_for_feature_not_root_no_hessians<T>(
	ordered_gradients: &[f32],
	binned_feature_values: &[T],
	bin_stats_for_feature: &mut [f64],
	examples_index: &[usize],
) where
	T: num_traits::cast::ToPrimitive,
{
	let unroll = NOT_ROOT_UNROLL;
	let len = examples_index.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			#[cfg(target_arch = "x86_64")]
			{
				let prefetch_index = *examples_index.get_unchecked(i + PREFETCH_OFFSET);
				let prefetch_ptr = binned_feature_values.as_ptr().add(prefetch_index) as *const i8;
				core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
			}
			let ordered_gradient = *ordered_gradients.get_unchecked(i);
			let example_index = *examples_index.get_unchecked(i);
			let bin_index = binned_feature_values
				.get_unchecked(example_index)
				.to_usize()
				.unwrap();
			let bin_index = bin_index << 1;
			*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
			*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += 1.0;
		}
	}
	for i in (len / unroll) * unroll..len {
		let ordered_gradient = *ordered_gradients.get_unchecked(i);
		let example_index = *examples_index.get_unchecked(i);
		let bin_index = binned_feature_values
			.get_unchecked(example_index)
			.to_usize()
			.unwrap();
		let bin_index = bin_index << 1;
		*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
		*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += 1.0;
	}
}

unsafe fn compute_bin_stats_for_feature_not_root<T>(
	ordered_gradients: &[f32],
	ordered_hessians: &[f32],
	binned_feature_values: &[T],
	bin_stats_for_feature: &mut [f64],
	examples_index: &[usize],
) where
	T: num_traits::cast::ToPrimitive,
{
	let unroll = NOT_ROOT_UNROLL;
	let len = examples_index.len();
	for i in 0..len / unroll {
		for i in i * unroll..i * unroll + unroll {
			#[cfg(target_arch = "x86_64")]
			{
				let prefetch_index = *examples_index.get_unchecked(i + PREFETCH_OFFSET);
				let prefetch_ptr = binned_feature_values.as_ptr().add(prefetch_index) as *const i8;
				core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
			}
			let ordered_gradient = *ordered_gradients.get_unchecked(i);
			let ordered_hessian = *ordered_hessians.get_unchecked(i);
			let example_index = *examples_index.get_unchecked(i);
			let bin_index = binned_feature_values
				.get_unchecked(example_index)
				.to_usize()
				.unwrap();
			let bin_index = bin_index << 1;
			*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
			*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += ordered_hessian as f64;
		}
	}
	for i in (len / unroll) * unroll..len {
		let ordered_gradient = *ordered_gradients.get_unchecked(i);
		let ordered_hessian = *ordered_hessians.get_unchecked(i);
		let example_index = *examples_index.get_unchecked(i);
		let bin_index = binned_feature_values
			.get_unchecked(example_index)
			.to_usize()
			.unwrap();
		let bin_index = bin_index << 1;
		*bin_stats_for_feature.get_unchecked_mut(bin_index) += ordered_gradient as f64;
		*bin_stats_for_feature.get_unchecked_mut(bin_index + 1) += ordered_hessian as f64;
	}
}

// Subtracts the bin_stats for a sibling from the parent.
// The subtraction method:
// 1. Compute the bin_stats for the child node with less examples.
// 2. Get the bin_stats for the child node with more examples by subtracting sibling_node_bin_stats from step 1 from the parent_bin_stats.
pub fn compute_bin_stats_subtraction(
	// (n_features, n_bins)
	parent_bin_stats: &mut BinStats,
	// (n_features, n_bins)
	sibling_bin_stats: &BinStats,
) {
	let iter = parent_bin_stats
		.entries
		.iter_mut()
		.zip(sibling_bin_stats.entries.iter());
	for (parent_bin_stats_for_feature, sibling_bin_stats_for_feature) in iter {
		compute_bin_stats_subtraction_for_feature(
			parent_bin_stats_for_feature,
			sibling_bin_stats_for_feature,
		);
	}
}

// Subtracts the sibling_bin_stats from the parent_bin_stats
// for a single feature.
fn compute_bin_stats_subtraction_for_feature(
	// (n_bins)
	parent_bin_stats_for_feature: &mut [f64],
	// (n_bins)
	sibling_bin_stats_for_feature: &[f64],
) {
	let iter = parent_bin_stats_for_feature
		.iter_mut()
		.zip(sibling_bin_stats_for_feature);
	for (parent_bin_stats, sibling_bin_stats) in iter {
		*parent_bin_stats -= sibling_bin_stats;
	}
}

/**
Returns the examples_index_ranges for the left and right nodes
and rearranges the examples_index so that the example indexes
in the first returned range are contained by the left node
and the example indexes in the second returned range
are contained by the right node.
*/
fn rearrange_examples_index(
	binned_features: &[BinnedFeaturesColumn],
	split: &SingleTreeBranchSplit,
	examples_index: &mut [usize],
	examples_index_left: &mut [usize],
	examples_index_right: &mut [usize],
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
	if examples_index.len() <= 1024 {
		rearrange_examples_index_serial(binned_features, split, examples_index)
	} else {
		rearrange_examples_index_parallel(
			binned_features,
			split,
			examples_index,
			examples_index_left,
			examples_index_right,
		)
	}
}

/// Rearrange examples index serially.
fn rearrange_examples_index_serial(
	binned_features: &[BinnedFeaturesColumn],
	split: &SingleTreeBranchSplit,
	examples_index: &mut [usize],
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
	let start = 0;
	let end = examples_index.len();
	let mut left = start;
	let mut right = end;
	let mut n_left = 0;
	while left < right {
		let direction = {
			match &split {
				SingleTreeBranchSplit::Continuous(SingleTreeBranchSplitContinuous {
					feature_index,
					bin_index,
					..
				}) => {
					let binned_feature = &binned_features[*feature_index];
					let feature_bin = match binned_feature {
						BinnedFeaturesColumn::U8(binned_feature) => {
							binned_feature[examples_index[left]].to_u8().unwrap()
						}
						BinnedFeaturesColumn::U16(binned_feature) => {
							binned_feature[examples_index[left]].to_u8().unwrap()
						}
					};
					if feature_bin <= *bin_index {
						SplitDirection::Left
					} else {
						SplitDirection::Right
					}
				}
				SingleTreeBranchSplit::Discrete(SingleTreeBranchSplitDiscrete {
					feature_index,
					directions,
					..
				}) => {
					let binned_feature = &binned_features[*feature_index];
					let feature_bin = match binned_feature {
						BinnedFeaturesColumn::U8(binned_feature) => {
							binned_feature[examples_index[left]].to_usize().unwrap()
						}
						BinnedFeaturesColumn::U16(binned_feature) => {
							binned_feature[examples_index[left]].to_usize().unwrap()
						}
					};
					if !directions.get(feature_bin).unwrap() {
						SplitDirection::Left
					} else {
						SplitDirection::Right
					}
				}
			}
		};
		match direction {
			SplitDirection::Left => {
				left += 1;
				n_left += 1;
			}
			SplitDirection::Right => {
				right -= 1;
				examples_index.swap(left, right);
			}
		};
	}
	(start..n_left, n_left..end)
}

/// Rearrange examples index in parallel.
fn rearrange_examples_index_parallel(
	binned_features: &[BinnedFeaturesColumn],
	split: &SingleTreeBranchSplit,
	examples_index: &mut [usize],
	examples_index_left: &mut [usize],
	examples_index_right: &mut [usize],
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
	let chunk_size = usize::max(examples_index.len() / 16, 1024);
	let counts: Vec<(usize, usize)> = izip!(
		ArrayViewMut1::from(&mut examples_index[..]).axis_chunks_iter(Axis(0), chunk_size),
		ArrayViewMut1::from(&mut examples_index_left[..]).axis_chunks_iter_mut(Axis(0), chunk_size),
		ArrayViewMut1::from(&mut examples_index_right[..])
			.axis_chunks_iter_mut(Axis(0), chunk_size),
	)
	.map(
		|(examples_index, mut examples_index_left, mut examples_index_right)| {
			// update left and right examples indexes and return n_left and n_right
			let mut n_left = 0;
			let mut n_right = 0;
			for example_index in examples_index {
				let direction = {
					match &split {
						SingleTreeBranchSplit::Continuous(SingleTreeBranchSplitContinuous {
							feature_index,
							bin_index,
							..
						}) => {
							let binned_features = &binned_features[*feature_index];
							let feature_bin = match binned_features {
								BinnedFeaturesColumn::U8(binned_features) => {
									binned_features[*example_index].to_u8().unwrap()
								}
								BinnedFeaturesColumn::U16(binned_features) => {
									binned_features[*example_index].to_u8().unwrap()
								}
							};
							if feature_bin <= *bin_index {
								SplitDirection::Left
							} else {
								SplitDirection::Right
							}
						}
						SingleTreeBranchSplit::Discrete(SingleTreeBranchSplitDiscrete {
							feature_index,
							directions,
							..
						}) => {
							let binned_features = &binned_features[*feature_index];
							let feature_bin = match binned_features {
								BinnedFeaturesColumn::U8(binned_features) => {
									binned_features[*example_index].to_usize().unwrap()
								}
								BinnedFeaturesColumn::U16(binned_features) => {
									binned_features[*example_index].to_usize().unwrap()
								}
							};
							if !directions.get(feature_bin).unwrap() {
								SplitDirection::Left
							} else {
								SplitDirection::Right
							}
						}
					}
				};
				match direction {
					SplitDirection::Left => {
						examples_index_left[n_left] = *example_index;
						n_left += 1;
					}
					SplitDirection::Right => {
						examples_index_right[n_right] = *example_index;
						n_right += 1;
					}
				}
			}
			(n_left, n_right)
		},
	)
	.collect();
	let mut left_starting_indexes: Vec<(usize, usize)> = Vec::with_capacity(counts.len());
	let mut left_starting_index = 0;
	for (n_left, _) in counts.iter() {
		left_starting_indexes.push((left_starting_index, *n_left));
		left_starting_index += n_left;
	}
	let mut right_starting_indexes: Vec<(usize, usize)> = Vec::with_capacity(counts.len());
	let mut right_starting_index = left_starting_index;
	for (_, n_right) in counts.iter() {
		right_starting_indexes.push((right_starting_index, *n_right));
		right_starting_index += n_right;
	}
	izip!(
		left_starting_indexes,
		right_starting_indexes,
		ArrayViewMut1::from(&mut examples_index_left[..]).axis_chunks_iter(Axis(0), chunk_size),
		ArrayViewMut1::from(&mut examples_index_right[..]).axis_chunks_iter(Axis(0), chunk_size),
	)
	.for_each(
		|(
			(left_starting_index, n_left),
			(right_starting_index, n_right),
			examples_index_left,
			examples_index_right,
		)| {
			let examples_index_slice =
				&examples_index[left_starting_index..left_starting_index + n_left];
			let examples_index_slice = unsafe {
				std::slice::from_raw_parts_mut(
					examples_index_slice.as_ptr() as *mut usize,
					examples_index_slice.len(),
				)
			};
			examples_index_slice
				.copy_from_slice(examples_index_left.slice(s![0..n_left]).to_slice().unwrap());
			let examples_index_slice =
				&examples_index[right_starting_index..right_starting_index + n_right];
			let examples_index_slice = unsafe {
				std::slice::from_raw_parts_mut(
					examples_index_slice.as_ptr() as *mut usize,
					examples_index_slice.len(),
				)
			};
			examples_index_slice.copy_from_slice(
				examples_index_right
					.slice(s![0..n_right])
					.to_slice()
					.unwrap(),
			);
		},
	);
	(
		0..left_starting_index,
		left_starting_index..examples_index.len(),
	)
}

pub struct FindSplitOutput {
	pub gain: f32,
	pub feature_index: usize,
	pub split: SingleTreeBranchSplit,
	pub left_sum_gradients: f64,
	pub left_sum_hessians: f64,
	pub left_n_examples: usize,
	pub right_sum_gradients: f64,
	pub right_sum_hessians: f64,
	pub right_n_examples: usize,
}

/// Find the split with the highest gain across all features, if a valid one exists.
fn find_split(
	bin_stats: &BinStats,
	sum_gradients: f64,
	sum_hessians: f64,
	examples_index_range: Range<usize>,
	options: &TrainOptions,
) -> Option<FindSplitOutput> {
	izip!(&bin_stats.entries, &bin_stats.binning_instructions)
		.enumerate()
		.filter_map(
			|(feature_index, (bin_stats, binning_instructions))| match binning_instructions {
				BinningInstructions::Number { .. } => {
					find_best_continuous_split_for_feature_left_to_right(
						feature_index,
						&binning_instructions,
						bin_stats,
						sum_gradients,
						sum_hessians,
						examples_index_range.clone(),
						options,
					)
				}
				BinningInstructions::Enum { .. } => {
					find_best_discrete_split_for_feature_left_to_right(
						feature_index,
						&binning_instructions,
						bin_stats,
						sum_gradients,
						sum_hessians,
						examples_index_range.clone(),
						options,
					)
				}
			},
		)
		.max_by(|a, b| a.gain.partial_cmp(&b.gain).unwrap())
}

/// Find the split with the highest gain across all features for both of the left and right child at the same time, if a valid one exists. A valid split will not exist if the split conditions are violated for all potential splits. By looping over the features once, we increase the cache efficiency.
#[allow(clippy::too_many_arguments)]
fn find_split_both(
	left_bin_stats: &BinStats,
	left_sum_gradients: f64,
	left_sum_hessians: f64,
	left_examples_index_range: Range<usize>,
	right_bin_stats: &BinStats,
	right_sum_gradients: f64,
	right_sum_hessians: f64,
	right_examples_index_range: Range<usize>,
	options: &TrainOptions,
) -> (Option<FindSplitOutput>, Option<FindSplitOutput>) {
	let best: Vec<(Option<FindSplitOutput>, Option<FindSplitOutput>)> =
		(0..left_bin_stats.entries.len())
			.map(|feature_index| {
				let binning_instructions = &left_bin_stats.binning_instructions[feature_index];
				match binning_instructions {
					BinningInstructions::Number { .. } => (
						find_best_continuous_split_for_feature_left_to_right(
							feature_index,
							binning_instructions,
							&left_bin_stats.entries[feature_index],
							left_sum_gradients,
							left_sum_hessians,
							left_examples_index_range.clone(),
							options,
						),
						find_best_continuous_split_for_feature_left_to_right(
							feature_index,
							binning_instructions,
							&right_bin_stats.entries[feature_index],
							right_sum_gradients,
							right_sum_hessians,
							right_examples_index_range.clone(),
							options,
						),
					),
					BinningInstructions::Enum { .. } => (
						find_best_discrete_split_for_feature_left_to_right(
							feature_index,
							&binning_instructions,
							&left_bin_stats.entries[feature_index],
							left_sum_gradients,
							left_sum_hessians,
							left_examples_index_range.clone(),
							options,
						),
						find_best_discrete_split_for_feature_left_to_right(
							feature_index,
							&binning_instructions,
							&right_bin_stats.entries[feature_index],
							right_sum_gradients,
							right_sum_hessians,
							right_examples_index_range.clone(),
							options,
						),
					),
				}
			})
			.collect();
	let (left, right) = best.into_iter().fold(
		(None, None),
		|a: (Option<FindSplitOutput>, Option<FindSplitOutput>),
		 b: (Option<FindSplitOutput>, Option<FindSplitOutput>)| {
			let left = match (a.0, b.0) {
				(Some(a), Some(b)) => {
					if a.gain > b.gain {
						Some(a)
					} else {
						Some(b)
					}
				}
				(Some(a), None) => Some(a),
				(None, Some(b)) => Some(b),
				(None, None) => None,
			};
			let right = match (a.1, b.1) {
				(Some(a), Some(b)) => {
					if a.gain > b.gain {
						Some(a)
					} else {
						Some(b)
					}
				}
				(Some(a), None) => Some(a),
				(None, Some(b)) => Some(b),
				(None, None) => None,
			};
			(left, right)
		},
	);
	(left, right)
}

/// Find the best split for this feature by iterating over the bins in sorted order, adding bins to the left tree and removing them from the right.
fn find_best_continuous_split_for_feature_left_to_right(
	feature_index: usize,
	binning_instructions: &BinningInstructions,
	bin_stats_for_feature: &[f64],
	sum_gradients_parent: f64,
	sum_hessians_parent: f64,
	examples_index_range: Range<usize>,
	options: &TrainOptions,
) -> Option<FindSplitOutput> {
	let negative_loss_parent_node = compute_negative_loss(
		sum_gradients_parent,
		sum_hessians_parent,
		options.l2_regularization,
	);
	let mut best_split_so_far: Option<FindSplitOutput> = None;
	let count_multiplier = examples_index_range.len() as f64 / sum_hessians_parent;
	let mut left_sum_gradients = 0.0;
	let mut left_sum_hessians = 0.0;
	let mut left_n_examples = 0;
	for (bin_index, bin_stats_entry) in bin_stats_for_feature.chunks(2).enumerate() {
		let sum_gradients = bin_stats_entry[0];
		let sum_hessians = bin_stats_entry[1];
		left_n_examples += (sum_hessians * count_multiplier)
			.round()
			.to_usize()
			.unwrap();
		left_sum_hessians += sum_hessians;
		left_sum_gradients += sum_gradients;
		let right_sum_gradients = sum_gradients_parent - left_sum_gradients;
		let right_sum_hessians = sum_hessians_parent - left_sum_hessians;
		let right_n_examples = match examples_index_range.len().checked_sub(left_n_examples) {
			Some(right_n_examples) => right_n_examples,
			None => break,
		};
		// check if we have violated the min samples leaf constraint
		if left_n_examples < options.min_examples_per_child {
			continue;
		}
		if right_n_examples < options.min_examples_per_child {
			// since we are in left to right mode, we will only get less examples if we continue so break instead
			break;
		}
		if left_sum_hessians < options.min_sum_hessians_per_child.to_f64().unwrap() {
			// Hessians are positive so the left sum hessians will continue to increase,
			// we can continue.
			continue;
		}
		if right_sum_hessians < options.min_sum_hessians_per_child.to_f64().unwrap() {
			// Hessians are positive so we will continue to violate the min_hessian_to_split
			// condition for the right node, break.
			break;
		}
		let current_split_gain = compute_gain(
			left_sum_gradients,
			left_sum_hessians,
			right_sum_gradients,
			right_sum_hessians,
			negative_loss_parent_node,
			options.l2_regularization,
		);
		// Figure out whether invalid values should go to the left subtree or to the right when predicting depending on whether the training dataset contains missing values or not.
		let invalid_values_direction = if bin_stats_for_feature[1] > 0.0 {
			// there are missing values in the training dataset and they have been added to the left subtree
			SplitDirection::Left
		} else {
			// there are no missing values in the training dataset. missing values should go to the branch with more examples
			if left_n_examples >= right_n_examples {
				SplitDirection::Left
			} else {
				SplitDirection::Right
			}
		};
		let split = SingleTreeBranchSplit::Continuous(SingleTreeBranchSplitContinuous {
			feature_index,
			bin_index: bin_index.to_u8().unwrap(),
			split_value: match binning_instructions {
				BinningInstructions::Number { thresholds } => {
					match bin_index.checked_sub(1) {
						Some(i) => thresholds[i],
						// its the null bucket
						None => f32::MIN,
					}
				}
				_ => unreachable!(),
			},
			invalid_values_direction,
		});
		let current_split = FindSplitOutput {
			feature_index,
			gain: current_split_gain,
			left_n_examples,
			left_sum_gradients,
			left_sum_hessians,
			right_n_examples,
			right_sum_gradients,
			right_sum_hessians,
			split,
		};
		match &best_split_so_far {
			Some(current_best_split) => {
				if current_split.gain > current_best_split.gain {
					best_split_so_far = Some(current_split);
				}
			}
			None => {
				if current_split.gain > options.min_gain_to_split {
					best_split_so_far = Some(current_split);
				}
			}
		}
	}
	best_split_so_far
}

/**
Find the best split for this discrete (categorical) feature. A discrete split is a partition of the categories into two subsets where one subset goes to the left subtree and one goes to the right.
To find the subsets:
1. Sort the bins by sum_gradients / (sum_hessians + categorical_smoothing_factor).
2. Perform the same algorithm to find the best split as the continuous setting, but iterate bins in the sorted order defined in step 1.
*/
fn find_best_discrete_split_for_feature_left_to_right(
	feature_index: usize,
	binning_instructions: &BinningInstructions,
	bin_stats_for_feature: &[f64],
	sum_gradients_parent: f64,
	sum_hessians_parent: f64,
	examples_index_range: Range<usize>,
	options: &TrainOptions,
) -> Option<FindSplitOutput> {
	let mut best_split_so_far: Option<FindSplitOutput> = None;
	let l2_regularization =
		options.l2_regularization + options.supplemental_l2_regularization_for_discrete_splits;
	let negative_loss_parent_node =
		compute_negative_loss(sum_gradients_parent, sum_hessians_parent, l2_regularization);
	let count_multiplier = examples_index_range.len() as f64 / sum_hessians_parent;
	let mut left_sum_gradients = 0.0;
	let mut left_sum_hessians = 0.0;
	let mut left_n_examples = 0;
	let smoothing_factor = options
		.smoothing_factor_for_discrete_bin_sorting
		.to_f64()
		.unwrap();
	let categorical_bin_score = |bin: &[f64]| bin[0] / (bin[1] + smoothing_factor);
	let mut sorted_bin_stats: Vec<(usize, &[f64])> =
		bin_stats_for_feature.chunks(2).enumerate().collect();
	sorted_bin_stats.sort_by(|(_, a), (_, b)| {
		categorical_bin_score(a)
			.partial_cmp(&categorical_bin_score(b))
			.unwrap()
	});
	let mut directions = vec![true; binning_instructions.n_valid_bins() + 1];
	for (bin_index, bin_stats_entry) in sorted_bin_stats.iter() {
		directions[*bin_index] = false;
		let sum_gradients = bin_stats_entry[0];
		let sum_hessians = bin_stats_entry[1];
		left_n_examples += (sum_hessians * count_multiplier)
			.round()
			.to_usize()
			.unwrap();
		left_sum_hessians += sum_hessians;
		left_sum_gradients += sum_gradients;
		let right_sum_gradients = sum_gradients_parent - left_sum_gradients;
		let right_sum_hessians = sum_hessians_parent - left_sum_hessians;
		let right_n_examples = match examples_index_range.len().checked_sub(left_n_examples) {
			Some(right_n_examples) => right_n_examples,
			None => break,
		};
		// check if we have violated the min samples leaf constraint
		if left_n_examples < options.min_examples_per_child {
			continue;
		}
		if right_n_examples < options.min_examples_per_child {
			// since we are in left to right mode, we will only get less examples if we continue so break instead
			break;
		}
		if left_sum_hessians < options.min_sum_hessians_per_child.to_f64().unwrap() {
			// Hessians are positive so the left sum hessians will continue to increase,
			// we can continue.
			continue;
		}
		if right_sum_hessians < options.min_sum_hessians_per_child.to_f64().unwrap() {
			// Hessians are positive so we will continue to violate the min_hessian_to_split
			// condition for the right node, break.
			break;
		}
		let current_split_gain = compute_gain(
			left_sum_gradients,
			left_sum_hessians,
			right_sum_gradients,
			right_sum_hessians,
			negative_loss_parent_node,
			l2_regularization,
		);
		// Figure out whether invalid values should go to the left subtree or to the right when predicting depending on whether the training dataset contains missing values or not.
		let invalid_values_direction = if bin_stats_for_feature[1] > 0.0 {
			// there are missing values in the training dataset and they have been added to the left subtree
			SplitDirection::Left
		} else {
			// there are no missing values in the training dataset. missing values should go to the branch with more examples
			if left_n_examples >= right_n_examples {
				SplitDirection::Left
			} else {
				SplitDirection::Right
			}
		};
		let split = SingleTreeBranchSplit::Discrete(SingleTreeBranchSplitDiscrete {
			feature_index,
			directions: directions.clone(),
		});
		let current_split = FindSplitOutput {
			feature_index,
			gain: current_split_gain,
			left_n_examples,
			left_sum_gradients,
			left_sum_hessians,
			right_n_examples,
			right_sum_gradients,
			right_sum_hessians,
			split,
		};
		match &best_split_so_far {
			Some(current_best_split) => {
				if current_split.gain > current_best_split.gain {
					best_split_so_far = Some(current_split)
				}
			}
			None => {
				if current_split.gain > options.min_gain_to_split {
					best_split_so_far = Some(current_split)
				}
			}
		}
	}
	best_split_so_far
}

/// The gain is a value that is used to measure how good a given split is.
#[inline(always)]
fn compute_gain(
	sum_gradients_left: f64,
	sum_hessians_left: f64,
	sum_gradients_right: f64,
	sum_hessians_right: f64,
	negative_loss_current_node: f32,
	l2_regularization: f32,
) -> f32 {
	let left = compute_negative_loss(sum_gradients_left, sum_hessians_left, l2_regularization);
	let right = compute_negative_loss(sum_gradients_right, sum_hessians_right, l2_regularization);
	left + right - negative_loss_current_node
}

/// The negative loss is used to compute the gain of a given split.
#[inline(always)]
fn compute_negative_loss(sum_gradients: f64, sum_hessians: f64, l2_regularization: f32) -> f32 {
	((sum_gradients * sum_gradients) / (sum_hessians + l2_regularization.to_f64().unwrap()))
		.to_f32()
		.unwrap()
}
