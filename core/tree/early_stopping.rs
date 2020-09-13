use super::{single, *};
use crate::dataframe::*;
use ndarray::prelude::*;
use num_traits::ToPrimitive;

#[derive(Clone)]
pub struct TrainStopMonitor {
	tolerance: f32,
	max_rounds_no_improve: usize,
	previous_stopping_metric: Option<f32>,
	num_rounds_no_improve: usize,
}

impl TrainStopMonitor {
	/// Create a train stop monitor
	pub fn new(tolerance: f32, max_rounds_no_improve: usize) -> Self {
		TrainStopMonitor {
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

pub fn train_early_stopping_split<'features, 'labels>(
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

pub fn compute_early_stopping_metrics(
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
