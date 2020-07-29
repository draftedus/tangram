use super::{
	early_stopping::{train_early_stopping_split, EarlyStoppingMonitor},
	shap, types,
};
use crate::{dataframe::*, metrics::*, util::super_unsafe::SuperUnsafe};
use ndarray::prelude::*;
use ndarray::Zip;
use rayon::prelude::*;

impl types::Regressor {
	pub fn train(
		features: ArrayView2<f32>,
		labels: &NumberColumnView,
		options: &types::TrainOptions,
	) -> Self {
		let n_features = features.ncols();
		let (features_train, labels_train, features_early_stopping, labels_early_stopping) =
			train_early_stopping_split(
				features,
				labels.data.into(),
				options.early_stopping_fraction,
			);
		let mut means = Vec::with_capacity(n_features);
		features_train
			.axis_iter(Axis(1))
			.into_par_iter()
			.map(|column| column.mean().unwrap())
			.collect_into_vec(&mut means);
		let mut model = Self {
			bias: 0.0,
			weights: Array1::<f32>::zeros(n_features),
			means: means.into(),
			losses: vec![].into(),
		};
		let mut early_stopping_monitor = if options.early_stopping_fraction > 0.0 {
			Some(EarlyStoppingMonitor::new())
		} else {
			None
		};
		for _ in 0..options.max_epochs {
			let model_cell = SuperUnsafe::new(model);
			(
				features_train.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
				labels_train.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
			)
				.into_par_iter()
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

	pub fn train_batch(
		&mut self,
		features: ArrayView2<f32>,
		labels: ArrayView1<f32>,
		options: &types::TrainOptions,
	) {
		let learning_rate = options.learning_rate;
		let predictions = features.dot(&self.weights) + self.bias;
		let py = (predictions - labels).insert_axis(Axis(1));
		let weight_gradients = (&features * &py).mean_axis(Axis(0)).unwrap();
		let bias_gradient: f32 = py.mean_axis(Axis(0)).unwrap()[0];
		Zip::from(self.weights.view_mut())
			.and(weight_gradients.view())
			.apply(|weight, weight_gradient| {
				*weight += -learning_rate * weight_gradient;
			});
		self.bias += -learning_rate * bias_gradient;
	}

	fn compute_early_stopping_metric_value(
		&self,
		features: ArrayView2<f32>,
		labels: ArrayView1<f32>,
		options: &types::TrainOptions,
	) -> f32 {
		(
			features.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
			labels.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
		)
			.into_par_iter()
			.fold(
				|| {
					let predictions =
						unsafe { <Array1<f32>>::uninitialized(options.n_examples_per_batch) };
					let metric = MeanSquaredError::default();
					(predictions, metric)
				},
				|mut state, (features, labels)| {
					let (predictions, metric) = &mut state;
					let slice = s![0..features.nrows()];
					let mut predictions = predictions.slice_mut(slice);
					self.predict(features, predictions.view_mut(), None);
					for (prediction, label) in predictions.iter().zip(labels.iter()) {
						metric.update((*prediction, *label));
					}
					state
				},
			)
			.map(|state| state.1)
			.reduce(MeanSquaredError::default, |mut metric, next_metric| {
				metric.merge(next_metric);
				metric
			})
			.finalize()
			.unwrap()
	}

	pub fn predict(
		&self,
		features: ArrayView2<f32>,
		mut predictions: ArrayViewMut1<f32>,
		mut shap_values: Option<ArrayViewMut3<f32>>,
	) {
		predictions.fill(self.bias);
		ndarray::linalg::general_mat_vec_mul(1.0, &features, &self.weights, 1.0, &mut predictions);

		if let Some(shap_values) = &mut shap_values {
			(
				features.axis_iter(Axis(0)),
				shap_values.axis_iter_mut(Axis(0)),
			)
				.into_par_iter()
				.for_each(|(features, mut shap_values)| {
					shap::compute_shap(
						features,
						self.bias,
						self.weights.view(),
						self.means.view(),
						shap_values.row_mut(0),
					);
				});
		}
	}
}
