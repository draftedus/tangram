use super::{
	early_stopping::{train_early_stopping_split, EarlyStoppingMonitor},
	shap, Progress, TrainOptions,
};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::ops::Neg;
use super_unsafe::SuperUnsafe;
use tangram_dataframe::*;
use tangram_metrics::{BinaryCrossEntropy, BinaryCrossEntropyInput, StreamingMetric};
use tangram_progress::ProgressCounter;

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryClassifier {
	pub weights: Array1<f32>,
	pub bias: f32,
	/// the mean of each feature value in the training set, which is used to compute SHAP values
	pub means: Vec<f32>,
	/// the loss value for each epoch
	pub losses: Vec<f32>,
	/// the class names of the target column
	pub classes: Vec<String>,
}

impl BinaryClassifier {
	pub fn train(
		features: ArrayView2<f32>,
		labels: &EnumColumnView,
		options: &TrainOptions,
		update_progress: &mut dyn FnMut(Progress),
	) -> BinaryClassifier {
		let n_features = features.ncols();
		let classes: Vec<String> = labels.options.to_vec();
		let (features_train, labels_train, features_early_stopping, labels_early_stopping) =
			train_early_stopping_split(
				features,
				labels.data.into(),
				options.early_stopping_fraction,
			);
		let means = features_train
			.axis_iter(Axis(1))
			.map(|column| column.mean().unwrap())
			.collect();
		let mut model = BinaryClassifier {
			bias: 0.0,
			weights: Array1::<f32>::zeros(n_features),
			means,
			losses: vec![],
			classes,
		};
		let mut early_stopping_monitor = if options.early_stopping_fraction > 0.0 {
			Some(EarlyStoppingMonitor::new())
		} else {
			None
		};
		let progress_counter = ProgressCounter::new(options.max_epochs.to_u64().unwrap());
		update_progress(Progress(progress_counter.clone()));
		for _ in 0..options.max_epochs {
			progress_counter.inc(1);
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

	pub fn train_batch(
		&mut self,
		features: ArrayView2<f32>,
		labels: ArrayView1<usize>,
		options: &TrainOptions,
	) {
		let learning_rate = options.learning_rate;
		let logits = features.dot(&self.weights) + self.bias;
		let mut predictions = logits.mapv_into(|logit| 1.0 / (logit.neg().exp() + 1.0));
		izip!(predictions.view_mut(), labels).for_each(|(prediction, label)| {
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
		izip!(self.weights.view_mut(), weight_gradients.view()).for_each(
			|(weight, weight_gradient)| {
				*weight += -learning_rate * weight_gradient;
			},
		);
		self.bias += -learning_rate * bias_gradient;
	}

	fn compute_early_stopping_metric_value(
		&self,
		features: ArrayView2<f32>,
		labels: ArrayView1<usize>,
		options: &TrainOptions,
	) -> f32 {
		izip!(
			features.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
			labels.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
		)
		.fold(
			{
				let predictions =
					unsafe { <Array2<f32>>::uninitialized((options.n_examples_per_batch, 2)) };
				let metric = BinaryCrossEntropy::default();
				(predictions, metric)
			},
			|mut state, (features, labels)| {
				let (predictions, metric) = &mut state;
				let slice = s![0..features.nrows(), ..];
				let mut predictions = predictions.slice_mut(slice);
				self.predict(features, predictions.view_mut());
				for (prediction, label) in predictions.column(1).iter().zip(labels.iter()) {
					metric.update(BinaryCrossEntropyInput {
						probability: *prediction,
						label: *label,
					});
				}
				state
			},
		)
		.1
		.finalize()
		.unwrap()
	}

	/// Write predicted probabilities into `probabilities` for the input `features`.
	pub fn predict(&self, features: ArrayView2<f32>, mut probabilities: ArrayViewMut2<f32>) {
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
		for probability_pos in probabilities_pos.iter_mut() {
			*probability_pos = 1.0 / (probability_pos.neg().exp() + 1.0);
		}
		for (neg, pos) in izip!(probabilities_neg.view_mut(), probabilities_pos.view()) {
			*neg = 1.0 - *pos;
		}
	}

	/// Write SHAP values into `shap_values` for the input `features`.
	pub fn compute_shap_values(
		&self,
		features: ArrayView2<f32>,
		mut shap_values: ArrayViewMut3<f32>,
	) {
		for (features, mut shap_values) in izip!(
			features.axis_iter(Axis(0)),
			shap_values.axis_iter_mut(Axis(0)),
		) {
			shap::compute_shap(
				features,
				self.bias,
				self.weights.view(),
				&self.means,
				shap_values.row_mut(0).as_slice_mut().unwrap(),
			);
		}
	}
}
