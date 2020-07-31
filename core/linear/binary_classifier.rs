use super::{
	early_stopping::{train_early_stopping_split, EarlyStoppingMonitor},
	shap, types,
};
use crate::{dataframe::*, metrics::*, util::super_unsafe::SuperUnsafe};
use ndarray::prelude::*;
use ndarray::Zip;
use rayon::prelude::*;
use std::ops::Neg;

impl types::BinaryClassifier {
	pub fn train(
		features: ArrayView2<f32>,
		labels: &EnumColumnView,
		options: &types::TrainOptions,
	) -> types::BinaryClassifier {
		let n_features = features.ncols();
		let classes: Vec<String> = labels.options.to_vec();
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
		let mut model = types::BinaryClassifier {
			bias: 0.0,
			weights: Array1::<f32>::zeros(n_features),
			means: means.into(),
			losses: vec![].into(),
			classes: classes.into(),
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
		labels: ArrayView1<usize>,
		options: &types::TrainOptions,
	) {
		let learning_rate = options.learning_rate;
		let logits = features.dot(&self.weights) + self.bias;
		let mut predictions = logits.mapv_into(|logit| 1.0 / (logit.neg().exp() + 1.0));
		Zip::from(predictions.view_mut())
			.and(labels)
			.apply(|prediction, label| {
				let label = match label {
					1 => 0.0,
					2 => 1.0,
					_ => unreachable!(),
				};
				*prediction -= label
			});
		let py = predictions.insert_axis(Axis(1));
		let weight_gradients = (&features * &py).mean_axis(Axis(0)).unwrap();
		let bias_gradient = py.mean_axis(Axis(0)).unwrap()[0];
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
		labels: ArrayView1<usize>,
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
						unsafe { <Array2<f32>>::uninitialized((options.n_examples_per_batch, 2)) };
					let metric = BinaryCrossEntropy::default();
					(predictions, metric)
				},
				|mut state, (features, labels)| {
					let (predictions, metric) = &mut state;
					let slice = s![0..features.nrows(), ..];
					let mut predictions = predictions.slice_mut(slice);
					self.predict(features, predictions.view_mut(), None);
					for (prediction, label) in predictions.column(1).iter().zip(labels.iter()) {
						metric.update(BinaryCrossEntropyInput {
							probability: *prediction,
							label: *label,
						});
					}
					state
				},
			)
			.map(|state| state.1)
			.reduce(BinaryCrossEntropy::default, |mut metric, next_metric| {
				metric.merge(next_metric);
				metric
			})
			.finalize()
			.unwrap()
	}

	pub fn predict(
		&self,
		features: ArrayView2<f32>,
		mut probabilities: ArrayViewMut2<f32>,
		mut shap_values: Option<ArrayViewMut3<f32>>,
	) {
		let mut probabilities_pos = probabilities.column_mut(1);
		probabilities_pos.fill(self.bias);
		ndarray::linalg::general_mat_vec_mul(
			1.0,
			&features,
			&self.weights,
			1.0,
			&mut probabilities_pos,
		);
		let (mut probabilities_neg, mut probabilities_pos) = probabilities.split_at(Axis(1), 1);
		Zip::from(probabilities_pos.view_mut())
			.apply(|probability_pos| *probability_pos = 1.0 / (probability_pos.neg().exp() + 1.0));
		Zip::from(probabilities_neg.view_mut())
			.and(probabilities_pos.view())
			.apply(|neg, pos| *neg = 1.0 - *pos);

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
