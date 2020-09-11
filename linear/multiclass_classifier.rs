use super::{
	early_stopping::{train_early_stopping_split, EarlyStoppingMonitor},
	shap, types,
};
use crate::{
	dataframe::*,
	metrics::{CrossEntropy, CrossEntropyInput, Metric},
	util::progress_counter::ProgressCounter,
	util::super_unsafe::SuperUnsafe,
};
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;

impl types::MulticlassClassifier {
	pub fn train(
		features: ArrayView2<f32>,
		labels: &EnumColumnView,
		options: &types::TrainOptions,
		update_progress: &mut dyn FnMut(super::Progress),
	) -> types::MulticlassClassifier {
		let n_classes = labels.options.len();
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
		let mut model = types::MulticlassClassifier {
			biases: Array1::<f32>::zeros(n_classes),
			weights: Array2::<f32>::zeros((n_features, n_classes)),
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
		update_progress(super::Progress(progress_counter.clone()));
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
		options: &types::TrainOptions,
	) {
		let learning_rate = options.learning_rate;
		let n_classes = self.weights.ncols();
		let mut logits = features.dot(&self.weights) + &self.biases;
		softmax(logits.view_mut());
		let mut predictions = logits;
		for (mut predictions, label) in izip!(predictions.genrows_mut(), labels) {
			for (class_index, prediction) in predictions.iter_mut().enumerate() {
				*prediction -= if class_index == label - 1 { 1.0 } else { 0.0 }
			}
		}
		let py = predictions;
		for class_index in 0..n_classes {
			let weight_gradients = (&features * &py.column(class_index).insert_axis(Axis(1)))
				.mean_axis(Axis(0))
				.unwrap();
			for (weight, weight_gradient) in izip!(
				self.weights.column_mut(class_index),
				weight_gradients.iter()
			) {
				*weight += -learning_rate * weight_gradient
			}
			let bias_gradients = py
				.column(class_index)
				.insert_axis(Axis(1))
				.mean_axis(Axis(0))
				.unwrap();
			self.biases[class_index] += -learning_rate * bias_gradients[0];
		}
	}

	fn compute_early_stopping_metric_value(
		&self,
		features: ArrayView2<f32>,
		labels: ArrayView1<usize>,
		options: &types::TrainOptions,
	) -> f32 {
		let n_classes = self.biases.len();
		izip!(
			features.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
			labels.axis_chunks_iter(Axis(0), options.n_examples_per_batch),
		)
		.fold(
			{
				let predictions = unsafe {
					<Array2<f32>>::uninitialized((options.n_examples_per_batch, n_classes))
				};
				let metric = CrossEntropy::default();
				(predictions, metric)
			},
			|mut state, (features, labels)| {
				let (predictions, metric) = &mut state;
				let slice = s![0..features.nrows(), ..];
				let mut predictions = predictions.slice_mut(slice);
				self.predict(features, predictions.view_mut(), None);
				for (prediction, label) in predictions.axis_iter(Axis(0)).zip(labels.iter()) {
					metric.update(CrossEntropyInput {
						probabilities: prediction,
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

	pub fn predict(
		&self,
		features: ArrayView2<f32>,
		mut probabilities: ArrayViewMut2<f32>,
		mut shap_values: Option<ArrayViewMut3<f32>>,
	) {
		let n_classes = probabilities.ncols();
		for mut row in probabilities.genrows_mut() {
			row.assign(&self.biases.view());
		}
		ndarray::linalg::general_mat_mul(1.0, &features, &self.weights, 1.0, &mut probabilities);
		softmax(probabilities);

		// compute shap
		if let Some(shap_values) = &mut shap_values {
			izip!(
				features.axis_iter(Axis(0)),
				shap_values.axis_iter_mut(Axis(0)),
			)
			.for_each(|(features, mut shap_values)| {
				for class_index in 0..n_classes {
					shap::compute_shap(
						features,
						self.biases[class_index],
						self.weights.row(class_index),
						&self.means,
						shap_values.row_mut(class_index),
					)
				}
			})
		}
	}
}

fn softmax(mut logits: ArrayViewMut2<f32>) {
	for mut logits in logits.genrows_mut() {
		let max = logits.iter().fold(std::f32::MIN, |a, &b| a.max(b));
		logits -= max;
		logits.mapv_inplace(|l| l.exp());
		let sum = logits.iter().fold(0.0, |a, b| a + b);
		logits /= sum;
	}
}
