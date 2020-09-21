use super::{train_tree::TrainTree, Task};
use ndarray::prelude::*;
use tangram_dataframe::*;

/// Compute the early stopping metric value for the set of trees that have been trained thus far.
pub fn compute_early_stopping_metric(
	task: &Task,
	trees: &[TrainTree],
	features: ArrayView2<Value>,
	labels: ColumnView,
	mut logits: ArrayViewMut2<f32>,
) -> f32 {
	match task {
		Task::Regression => {
			let labels = labels.as_number().unwrap().data.into();
			crate::regressor::update_logits(trees, features.view(), logits.view_mut());
			crate::regressor::compute_loss(labels, logits.view())
		}
		Task::BinaryClassification => {
			let labels = labels.as_enum().unwrap().data.into();
			crate::binary_classifier::update_logits(trees, features.view(), logits.view_mut());
			crate::binary_classifier::compute_loss(labels, logits.view())
		}
		Task::MulticlassClassification { .. } => {
			let labels = labels.as_enum().unwrap().data.into();
			crate::multiclass_classifier::update_logits(trees, features.view(), logits.view_mut());
			crate::multiclass_classifier::compute_loss(labels, logits.view())
		}
	}
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
