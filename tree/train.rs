use crate::{
	bin::{
		compute_bin_info, compute_binned_features, filter_binned_features, ComputeBinInfoOptions,
		FilterBinnedFeaturesOptions,
	},
	bin_stats::BinStatsPool,
	feature_importances::compute_feature_importances,
	single,
	timing::Timing,
	BinaryClassifier, Model, MulticlassClassifier, Progress, Regressor, Task, TrainOptions, Tree,
};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use super_unsafe::SuperUnsafe;
use tangram_dataframe::*;
use tangram_progress::ProgressCounter;

/// To avoid code duplication, this shared `train` method is called by `Regressor::train`, `BinaryClassifier::train`, and `MulticlassClassifier::train`.
pub fn train(
	task: &Task,
	features: DataFrameView,
	labels: ColumnView,
	options: TrainOptions,
	update_progress: &mut dyn FnMut(Progress),
) -> Model {
	// let timing = Timing::new();

	// determine how to bin each column
	let bin_options = ComputeBinInfoOptions {
		max_valid_bins: options.max_non_missing_bins,
		max_number_column_examples_for_bin_info: options.subsample_for_binning,
	};
	let bin_info = compute_bin_info(&features, &bin_options);

	// compute the binned values
	let n_bins = options.max_non_missing_bins as usize + 1;
	let progress_counter = ProgressCounter::new(features.nrows().to_u64().unwrap());
	update_progress(super::Progress::Initializing(progress_counter.clone()));
	let (features, features_stats) =
		compute_binned_features(&features, &bin_info, n_bins as usize, &|| {
			progress_counter.inc(1)
		});

	// TODO fold this step into compute_bin_info and compute_binned_features
	let filter_options = FilterBinnedFeaturesOptions {
		min_examples_split: options.min_examples_leaf,
	};
	let include_features =
		filter_binned_features(features.view(), features_stats, &bin_info, filter_options);

	// if early stopping is enabled then split the features and labels into train and early stopping sets.
	let early_stopping_enabled = options.early_stopping_options.is_some();
	let (
		features_train,
		labels_train,
		features_early_stopping,
		labels_early_stopping,
		mut train_stop_monitor,
	) = match &options.early_stopping_options {
		Some(options) => {
			let (features_train, labels_train, features_early_stopping, labels_early_stopping) =
				train_early_stopping_split(
					features.view(),
					labels,
					options.early_stopping_fraction,
				);
			let train_stop_monitor = EarlyStoppingMonitor::new(
				options.early_stopping_threshold,
				options.early_stopping_rounds,
			);
			(
				features_train,
				labels_train,
				Some(features_early_stopping),
				Some(labels_early_stopping),
				Some(train_stop_monitor),
			)
		}
		None => (features.view(), labels, None, None, None),
	};

	let n_examples = features_train.nrows();
	let n_features = features_train.ncols();

	// Regression and binary classification have one tree for each round. Multiclass classification has one tree per class for each round.
	let n_trees_per_round = match task {
		Task::Regression => 1,
		Task::BinaryClassification => 1,
		Task::MulticlassClassification { n_trees_per_round } => *n_trees_per_round,
	};

	// The mean square error loss used in regression has a constant second derivative, so there is no need to update hessians for regression tasks.
	let has_constant_hessians = match task {
		Task::Regression => true,
		Task::BinaryClassification => false,
		Task::MulticlassClassification { .. } => false,
	};

	// A tree model's prediction will be a bias plus the sum of the outputs of each tree. The bias will produce the baseline prediction.
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
	let mut predictions = unsafe { Array::uninitialized((n_trees_per_round, n_examples)) };
	let mut gradients = unsafe { Array::uninitialized((n_trees_per_round, n_examples)) };
	let mut hessians = unsafe { Array::uninitialized((n_trees_per_round, n_examples)) };
	let mut ordered_gradients = unsafe { Array::uninitialized((n_trees_per_round, n_examples)) };
	let mut ordered_hessians = unsafe { Array::uninitialized((n_trees_per_round, n_examples)) };
	let mut examples_index = unsafe { Array::uninitialized((n_trees_per_round, n_examples)) };
	let mut examples_index_left = unsafe { Array::uninitialized((n_trees_per_round, n_examples)) };
	let mut examples_index_right = unsafe { Array::uninitialized((n_trees_per_round, n_examples)) };
	let mut bin_stats_pools: Vec<BinStatsPool> =
		vec![BinStatsPool::new(options.max_leaf_nodes, &bin_info); n_trees_per_round];
	let mut logits_early_stopping = if early_stopping_enabled {
		Some(unsafe {
			Array::uninitialized((n_trees_per_round, features_early_stopping.unwrap().nrows()))
		})
	} else {
		None
	};

	// This is the total number of rounds that have been trained thus far.
	let mut n_rounds_trained = 0;
	// These are the trees in round-major order. After training this will have shape (n_rounds, n_trees_per_round).
	let mut trees: Vec<single::TrainTree> = Vec::new();
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

	// Train rounds of trees until we hit max_rounds or the EarlyStoppingMonitor indicates we should stop early.
	let round_counter = ProgressCounter::new(options.max_rounds.to_u64().unwrap());
	update_progress(super::Progress::Training(round_counter.clone()));
	for round_index in 0..options.max_rounds {
		round_counter.inc(1);
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
				let (tree, leaf_values) = single::train(
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
					let predictions_cell = SuperUnsafe::new(predictions.as_slice_mut().unwrap());
					leaf_values.iter().for_each(|(range, value)| {
						examples_index.as_slice().unwrap()[range.clone()]
							.iter()
							.for_each(|&example_index| {
								let predictions = unsafe { predictions_cell.get() };
								predictions[example_index] += value;
							});
					});
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

		// If early stopping is enabled, compute the early stopping metrics and update the train stop monitor to see if we should stop training at this round.
		if early_stopping_enabled {
			let features_early_stopping = features_early_stopping.unwrap();
			let labels_early_stopping = labels_early_stopping.as_ref().unwrap();
			let logits_early_stopping = logits_early_stopping.as_mut().unwrap();
			let train_stop_monitor = train_stop_monitor.as_mut().unwrap();
			for mut logits in logits_early_stopping.gencolumns_mut() {
				logits.assign(&biases);
			}
			let value = compute_early_stopping_metric(
				&task,
				trees_for_round.as_slice(),
				features_early_stopping,
				labels_early_stopping.view(),
				logits_early_stopping.view_mut(),
			);
			let should_stop = train_stop_monitor.update(value);
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

fn train_early_stopping_split<'features, 'labels>(
	features: ArrayView2<'features, u8>,
	labels: ColumnView<'labels>,
	early_stopping_fraction: f32,
) -> (
	ArrayView2<'features, u8>,
	ColumnView<'labels>,
	ArrayView2<'features, u8>,
	ColumnView<'labels>,
) {
	let split_index = (early_stopping_fraction * features.nrows().to_f32().unwrap())
		.to_usize()
		.unwrap();
	let (features_early_stopping, features_train) = features.split_at(Axis(0), split_index);
	let (labels_early_stopping, labels_train) = labels.split_at_row(split_index);
	(
		features_train,
		labels_train,
		features_early_stopping,
		labels_early_stopping,
	)
}

#[derive(Clone)]
pub struct EarlyStoppingMonitor {
	tolerance: f32,
	max_rounds_no_improve: usize,
	previous_stopping_metric: Option<f32>,
	num_rounds_no_improve: usize,
}

impl EarlyStoppingMonitor {
	/// Create a train stop monitor
	pub fn new(tolerance: f32, max_rounds_no_improve: usize) -> Self {
		EarlyStoppingMonitor {
			tolerance,
			max_rounds_no_improve,
			previous_stopping_metric: None,
			num_rounds_no_improve: 0,
		}
	}

	/// Update with the next epoch's task metrics. Returns true if training should stop
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

fn compute_early_stopping_metric(
	task: &Task,
	trees: &[single::TrainTree],
	features: ArrayView2<u8>,
	labels: ColumnView,
	mut logits: ArrayViewMut2<f32>,
) -> f32 {
	match task {
		Task::Regression => {
			let labels = labels.as_number().unwrap().data.into();
			super::regressor::update_logits(trees, features, logits.view_mut());
			super::regressor::compute_loss(labels, logits.view())
		}
		Task::BinaryClassification => {
			let labels = labels.as_enum().unwrap().data.into();
			super::binary_classifier::update_logits(trees, features, logits.view_mut());
			super::binary_classifier::compute_loss(labels, logits.view())
		}
		Task::MulticlassClassification { .. } => {
			let labels = labels.as_enum().unwrap().data.into();
			super::multiclass_classifier::update_logits(trees, features, logits.view_mut());
			super::multiclass_classifier::compute_loss(labels, logits.view())
		}
	}
}
