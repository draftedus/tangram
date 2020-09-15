use ndarray::prelude::*;
use num_traits::ToPrimitive;

pub fn train_early_stopping_split<'features, 'labels, Label>(
	features: ArrayView2<'features, f32>,
	labels: ArrayView1<'labels, Label>,
	early_stopping_fraction: f32,
) -> (
	ArrayView2<'features, f32>,
	ArrayView1<'labels, Label>,
	ArrayView2<'features, f32>,
	ArrayView1<'labels, Label>,
) {
	let split_index = ((1.0 - early_stopping_fraction) * features.nrows().to_f32().unwrap())
		.to_usize()
		.unwrap();
	let (features_train, features_early_stopping) = features.split_at(Axis(0), split_index);
	let (labels_train, labels_early_stopping) = labels.split_at(Axis(0), split_index);
	(
		features_train,
		labels_train,
		features_early_stopping,
		labels_early_stopping,
	)
}

const TOLERANCE: f32 = 0.0001;
const NUM_ROUNDS_NO_IMPROVE: usize = 5;

#[derive(Clone)]
pub struct EarlyStoppingMonitor {
	previous_stopping_metric_value: Option<f32>,
	n_rounds_no_improve: usize,
}

impl EarlyStoppingMonitor {
	// Create a new EarlyStoppingMonitor.
	pub fn new() -> Self {
		EarlyStoppingMonitor {
			previous_stopping_metric_value: None,
			n_rounds_no_improve: 0,
		}
	}

	/// Update with the next epoch's task metrics. Returns true if training should stop
	pub fn update(&mut self, value: f32) -> bool {
		let stopping_metric = value;
		let result = if let Some(previous_stopping_metric) = self.previous_stopping_metric_value {
			if stopping_metric > previous_stopping_metric
				|| f32::abs(stopping_metric - previous_stopping_metric) < TOLERANCE
			{
				self.n_rounds_no_improve += 1;
				self.n_rounds_no_improve >= NUM_ROUNDS_NO_IMPROVE
			} else {
				self.n_rounds_no_improve = 0;
				false
			}
		} else {
			false
		};
		self.previous_stopping_metric_value = Some(stopping_metric);
		result
	}
}
