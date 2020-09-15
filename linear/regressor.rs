use super::{
	compute_shap_values_common, train_early_stopping_split, EarlyStoppingMonitor, TrainOptions,
};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use super_unsafe::SuperUnsafe;
use tangram_dataframe::*;
use tangram_metrics::{MeanSquaredError, StreamingMetric};
use tangram_progress::ProgressCounter;

/// This struct describes a linear regressor model. You can train one by calling `Regressor::train`.
#[derive(Debug)]
pub struct Regressor {
	pub bias: f32,
	pub weights: Array1<f32>,
	/// These are the mean values of each feature in the training set, which are used to compute SHAP values.
	pub means: Vec<f32>,
	/// These are the loss values for each epoch.
	pub losses: Vec<f32>,
}

impl Regressor {
	/// Train a linear regressor.
	pub fn train(
		features: ArrayView2<f32>,
		labels: &NumberColumnView,
		options: &TrainOptions,
		update_progress: &mut dyn FnMut(super::Progress),
	) -> Self {
		let n_features = features.ncols();
		let (features_train, labels_train, features_early_stopping, labels_early_stopping) =
			train_early_stopping_split(
				features,
				labels.data.into(),
				options
					.early_stopping_options
					.as_ref()
					.map(|o| o.early_stopping_fraction)
					.unwrap_or(0.0),
			);
		let means = features_train
			.axis_iter(Axis(1))
			.map(|column| column.mean().unwrap())
			.collect();
		let mut model = Self {
			bias: 0.0,
			weights: Array1::<f32>::zeros(n_features),
			means,
			losses: vec![],
		};
		let mut early_stopping_monitor =
			if let Some(early_stopping_options) = &options.early_stopping_options {
				Some(EarlyStoppingMonitor::new(
					early_stopping_options.min_decrease_in_loss_for_significant_change,
					early_stopping_options.n_epochs_without_improvement_to_stop,
				))
			} else {
				None
			};
		let epoch_counter = ProgressCounter::new(options.max_epochs.to_u64().unwrap());
		update_progress(super::Progress(epoch_counter.clone()));
		for _ in 0..options.max_epochs {
			epoch_counter.inc(1);
			let model_cell = SuperUnsafe::new(model);
			izip!(
				features_train.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
				labels_train.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
			)
			.for_each(|(features, labels)| {
				let model = unsafe { model_cell.get() };
				Self::train_batch(model, features, labels, options);
			});
			model = model_cell.into_inner();
			if let Some(early_stopping_monitor) = early_stopping_monitor.as_mut() {
				let early_stopping_metric_value = Self::compute_early_stopping_metric_value(
					&model,
					features_early_stopping,
					labels_early_stopping,
					options,
				);
				let should_stop = early_stopping_monitor.update(early_stopping_metric_value);
				if should_stop {
					break;
				}
			}
		}
		model
	}

	fn train_batch(
		&mut self,
		features: ArrayView2<f32>,
		labels: ArrayView1<f32>,
		options: &TrainOptions,
	) {
		let learning_rate = options.learning_rate;
		let predictions = features.dot(&self.weights) + self.bias;
		let py = (predictions - labels).insert_axis(Axis(1));
		let weight_gradients = (&features * &py).mean_axis(Axis(0)).unwrap();
		let bias_gradient: f32 = py.mean_axis(Axis(0)).unwrap()[0];
		for (weight, weight_gradient) in izip!(self.weights.iter_mut(), weight_gradients.iter()) {
			*weight += -learning_rate * weight_gradient;
		}
		self.bias += -learning_rate * bias_gradient;
	}

	fn compute_early_stopping_metric_value(
		&self,
		features: ArrayView2<f32>,
		labels: ArrayView1<f32>,
		options: &TrainOptions,
	) -> f32 {
		izip!(
			features.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
			labels.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
		)
		.fold(
			{
				let predictions =
					unsafe { <Array1<f32>>::uninitialized(options.n_examples_per_batch) };
				let metric = MeanSquaredError::default();
				(predictions, metric)
			},
			|mut state, (features, labels)| {
				let (predictions, metric) = &mut state;
				let slice = s![0..features.nrows()];
				let mut predictions = predictions.slice_mut(slice);
				self.predict(features, predictions.view_mut());
				for (prediction, label) in predictions.iter().zip(labels.iter()) {
					metric.update((*prediction, *label));
				}
				state
			},
		)
		.1
		.finalize()
		.unwrap()
	}

	/// Write predictions into `predictions` for the input `features`.
	pub fn predict(&self, features: ArrayView2<f32>, mut predictions: ArrayViewMut1<f32>) {
		predictions.fill(self.bias);
		ndarray::linalg::general_mat_vec_mul(1.0, &features, &self.weights, 1.0, &mut predictions);
	}

	/// Write SHAP values into `shap_values` for the input `features`.
	pub fn compute_shap_values(
		&self,
		features: ArrayView2<f32>,
		mut shap_values: ArrayViewMut3<f32>,
	) {
		izip!(
			features.axis_iter(Axis(0)),
			shap_values.axis_iter_mut(Axis(0)),
		)
		.for_each(|(features, mut shap_values)| {
			compute_shap_values_common(
				features,
				self.bias,
				self.weights.view(),
				&self.means,
				shap_values.row_mut(0).as_slice_mut().unwrap(),
			);
		});
	}
}
